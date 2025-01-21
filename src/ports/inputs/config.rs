/**
 * Trait for configuration management, providing methods to load and save configurations.
 *
 * This trait defines the necessary operations for handling configuration data,
 * including loading from and saving to a specified path. It uses an associated type
 * for configuration and input, allowing flexibility in how configurations are managed.
 */
pub trait Config: Sized {
    type Error;
    type Config;
    type Input;
    /// This would be the default path for the configuration file.
    const PATH: &'static str = "config.yaml";
    /// Loads the configuration from a specified input source.
    ///
    /// # Arguments
    ///
    /// * `input` - An optional input source for loading the configuration.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Config, Self::Error>` - The loaded configuration wrapped in a `Result`.
    async fn load(input: Option<Self::Input>) -> Result<Self::Config, Self::Error>;
    /// Saves the configuration to a specified input source.
    ///
    /// # Arguments
    ///
    /// * `input` - An optional input source for saving the configuration.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Config, Self::Error>` - The result of the save operation wrapped in a `Result`.
    async fn save(input: Option<Self::Input>) -> Result<Self::Config, Self::Error>;
}