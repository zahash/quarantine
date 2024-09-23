use anyhow::anyhow;
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogOutput, StartContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use bollard::secret::{ErrorDetail, HostConfig};
use bollard::Docker;
use clap::Parser;
use futures::StreamExt;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let Quarantine {
        image_name,
        persist: _,
        runtime,
    } = Quarantine::parse();

    let docker = Docker::connect_with_local_defaults()?;
    let info = docker.info().await?;
    let default_runtime = info.default_runtime.unwrap_or_default();
    let available_runtimes = info.runtimes.unwrap_or_default();

    let runtime = match runtime {
        Some(runtime) => match available_runtimes.contains_key(&runtime) {
            true => {
                tracing::info!("using runtime `{}`", runtime);
                runtime
            }
            false => {
                tracing::warn!(
                    "runtime `{}` not found! reverting to the default `{}`",
                    runtime,
                    default_runtime
                );
                tracing::warn!(
                    "available runtimes are {}",
                    available_runtimes
                        .keys()
                        .into_iter()
                        .map(|key| format!("`{}`", key))
                        .collect::<Vec<String>>()
                        .join(" ")
                );
                default_runtime
            }
        },
        None => {
            tracing::info!("using default runtime `{}`", default_runtime);
            default_runtime
        }
    };

    // pull image
    {
        let mut stream = docker.create_image(
            Some(CreateImageOptions {
                from_image: image_name.as_str(),
                ..Default::default()
            }),
            None,
            None,
        );

        tracing::info!("pulling image: {}", image_name);
        while let Some(Ok(pull_result)) = stream.next().await {
            if let Some(error) = pull_result.error {
                tracing::error!("{}", error);
                if let Some(ErrorDetail {
                    code: Some(code),
                    message: Some(message),
                }) = pull_result.error_detail
                {
                    tracing::error!("{} :: {}", code, message);
                }
            } else {
                tracing::info!(
                    "{} {} {}",
                    pull_result.id.unwrap_or_default(),
                    pull_result.status.unwrap_or_default(),
                    pull_result.progress.unwrap_or_default(),
                );
            }
        }
    }

    let container_name = format!("quarantine-{}", image_name.replace(":", "-"));

    // stop and remove any previously running containers
    {
        let list_containers_options: ListContainersOptions<String> = ListContainersOptions {
            all: true,
            ..Default::default()
        };

        let containers = docker
            .list_containers(Some(list_containers_options))
            .await?;

        tracing::info!(
            "checking for any previously running containers with the name: {}",
            container_name
        );
        for container in containers {
            for name in container.names.unwrap_or_default() {
                if name.trim_start_matches("/") == container_name {
                    if let Some(state) = &container.state {
                        if state.to_lowercase() == "running" {
                            tracing::info!("stopping running container: {}", &container_name);
                            docker.stop_container(&container_name, None).await?;
                        }
                        tracing::info!("removing container: {}", &container_name);
                        docker.remove_container(&container_name, None).await?;
                    }
                }
            }
        }
    }

    // start container
    {
        let options = Some(CreateContainerOptions {
            name: container_name.as_str(),
            ..Default::default()
        });

        let mut volumes = HashMap::new();
        volumes.insert("/quarantine".to_string(), HashMap::new());

        let current_dir = std::env::current_dir()?
            .into_os_string()
            .into_string()
            .map_err(|_| anyhow!("current working directory path is not valid unicode"))?;

        let host_config = HostConfig {
            runtime: Some(runtime),
            binds: Some(vec![format!("{}:/quarantine", current_dir)]),
            ..Default::default()
        };

        let config = Config {
            image: Some(image_name),
            tty: Some(true),
            working_dir: Some("/quarantine".into()),
            volumes: Some(volumes),
            host_config: Some(host_config),
            ..Default::default()
        };

        let container = docker.create_container(options, config).await?;
        tracing::info!(
            "starting new container: {} :: name: {}",
            container.id,
            container_name
        );
        docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await?;
        tracing::info!(
            "container started: {} :: name: {}",
            container.id,
            container_name
        );
    };

    {
        tracing::info!("creating an exec instance to run a shell in the container");
        let create_exec = docker
            .create_exec(
                &container_name,
                CreateExecOptions {
                    attach_stdin: Some(true),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    tty: Some(true),
                    cmd: Some(vec!["sh", "-c", "stty -echo; exec sh"]),
                    ..Default::default()
                },
            )
            .await?;

        let start_exec = docker
            .start_exec(
                &create_exec.id,
                Some(StartExecOptions {
                    detach: false,
                    tty: true,
                    output_capacity: None,
                }),
            )
            .await?;

        let StartExecResults::Attached {
            mut output,
            mut input,
        } = start_exec
        else {
            return Err(anyhow!("failed to execute shell inside container"));
        };

        let mut stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut stderr = tokio::io::stderr();

        tracing::info!("redirecting inputs and outputs");

        let input_fut = async {
            // copy stdin to container input
            let mut input_buffer = vec![0; 1024];
            loop {
                let bytes_read = stdin.read(&mut input_buffer).await?;
                if bytes_read == 0 {
                    tracing::info!("EOF reached on stdin");
                    break;
                }
                input.write_all(&input_buffer[..bytes_read]).await?;
            }
            Ok::<_, bollard::errors::Error>(())
        };

        let output_fut = async {
            // copy container output to stdout
            while let Some(output) = output.next().await {
                match output {
                    Ok(LogOutput::StdOut { message }) => stdout.write_all(&message).await?,
                    Ok(LogOutput::StdErr { message }) => stderr.write_all(&message).await?,
                    Ok(LogOutput::Console { message }) => stdout.write_all(&message).await?,
                    Err(e) => tracing::error!("error reading output: {:?}", e),
                    other => tracing::info!("{:?}", other),
                }
                stdout.flush().await?;
                stderr.flush().await?;
            }
            Ok::<_, bollard::errors::Error>(())
        };

        tokio::select! {
            _ = tokio::signal::ctrl_c() => { /* catch ctrl_c */  }
            result = input_fut => { result? }
            result = output_fut => { result? }
        };
    }

    // Stop and clean up the container after use
    {
        tracing::info!("stopping container: {}", container_name);
        docker.stop_container(&container_name, None).await?;

        tracing::info!("removing container: {}", container_name);
        docker.remove_container(&container_name, None).await?;
    }

    tracing::info!("done");
    Ok(())
}

#[derive(Parser, Debug)]
struct Quarantine {
    /// image name with (optional)tag. eg: `python:latest` or `golang` or `node:20.17.0` or `node:20.17.0-alpine3.19`
    #[arg(short, long)]
    image_name: String,

    /// which container runtime to use (eg: `runsc`). will revert to the default runtime if the one specified is not found.
    #[arg(short, long)]
    runtime: Option<String>,

    /// persist container after use
    #[arg(short, long, default_value_t = false)]
    persist: bool,
}
