use async_trait::async_trait;
use dirs_next::home_dir;
use ini::Ini;
use rusoto_core::credential::{
    AwsCredentials, ContainerProvider, CredentialsError, EnvironmentProvider,
    InstanceMetadataProvider, ProfileProvider, ProvideAwsCredentials,
};
use std::env;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct AwsCredentialsProvider {
    prefixed_environment_provider: EnvironmentProvider,
    environment_provider: EnvironmentProvider,
    profile_provider: Option<ProfileProvider>,
    container_provider: Option<ContainerProvider>,
    instance_metadata_provider: Option<InstanceMetadataProvider>,
}

impl AwsCredentialsProvider {
    pub fn new() -> AwsCredentialsProvider {
        // Set up profile provider using optionally supplied profile name //
        let profile_provider: Option<ProfileProvider>;
        if let Ok(profile_name) = env::var("MANTLE_AWS_PROFILE") {
            let mut provider = ProfileProvider::new().unwrap();
            provider.set_profile(profile_name);
            profile_provider = Some(provider);
        } else {
            profile_provider = ProfileProvider::new().ok();
        }

        // Inherit IAM role from instance metadata service or ECS agent role //
        let mut inherit_iam_role = false;
        if let Ok(value) = env::var("MANTLE_AWS_INHERIT_IAM_ROLE") {
            if value == "true" {
                inherit_iam_role = true;
            }
        }

        AwsCredentialsProvider {
            prefixed_environment_provider: EnvironmentProvider::with_prefix("MANTLE_AWS"),
            environment_provider: EnvironmentProvider::default(),
            profile_provider,
            container_provider: if inherit_iam_role {
                let mut provider = ContainerProvider::new();
                provider.set_timeout(Duration::from_secs(15));
                Some(provider)
            } else {
                None
            },
            instance_metadata_provider: if inherit_iam_role {
                let mut provider = InstanceMetadataProvider::new();
                provider.set_timeout(Duration::from_secs(15));
                Some(provider)
            } else {
                None
            },
        }
    }
}

fn get_config_path() -> PathBuf {
    home_dir()
        .expect("Expected a HOME directory")
        .join(".aws")
        .join("config")
}

async fn chain_provider_credentials(
    provider: AwsCredentialsProvider,
) -> Result<AwsCredentials, CredentialsError> {
    if let Ok(creds) = provider.prefixed_environment_provider.credentials().await {
        return Ok(creds);
    }
    if let Ok(creds) = provider.environment_provider.credentials().await {
        return Ok(creds);
    }
    if let Some(ref profile_provider) = provider.profile_provider {
        // Check standard profile credentials first //
        if let Ok(creds) = profile_provider.credentials().await {
            return Ok(creds);
        }

        // Check SSO profile credentials as fallback //
        println!("Checking profile provider (sso)");
        let aws_config = Ini::load_from_file(get_config_path())
            .expect(format!("Failed to load AWS config ({:?})", get_config_path()).as_str());
        let profile_name = profile_provider.profile();
        println!("profile name: {}", profile_name);

        let target_section = aws_config.iter().find(|(section, _)| {
            section.is_some() && section.unwrap() == format!("profile {}", profile_name)
        });

        if let Some((section, properties)) = target_section {
            let section_name = section.unwrap();
            println!("Section name: {}", section_name);
            for (key, value) in properties.iter() {
                println!("{}: {:?}", key, value);
            }
        }
    }
    if let Some(ref container_provider) = provider.container_provider {
        if let Ok(creds) = container_provider.credentials().await {
            return Ok(creds);
        }
    }
    if let Some(ref instance_metadata_provider) = provider.instance_metadata_provider {
        if let Ok(creds) = instance_metadata_provider.credentials().await {
            return Ok(creds);
        }
    }
    Err(CredentialsError::new(
        "Couldn't find AWS credentials in environment, credentials file, or instance/container IAM role.",
    ))
}

#[async_trait]
impl ProvideAwsCredentials for AwsCredentialsProvider {
    async fn credentials(&self) -> Result<AwsCredentials, CredentialsError> {
        chain_provider_credentials(self.clone()).await
    }
}
