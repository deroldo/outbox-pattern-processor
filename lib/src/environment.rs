use std::env;

pub struct Environment;

impl Environment {
    pub fn string(
        env_name: &str,
        default: &str,
    ) -> String {
        env::var(env_name).ok().unwrap_or(default.to_string())
    }

    pub fn u16(
        env_name: &str,
        default: u16,
    ) -> u16 {
        env::var(env_name).ok().map(|env| env.parse::<u16>().expect("Failed to parse to u16")).unwrap_or(default)
    }

    pub fn u32(
        env_name: &str,
        default: u32,
    ) -> u32 {
        env::var(env_name).ok().map(|env| env.parse::<u32>().expect("Failed to parse to u32")).unwrap_or(default)
    }

    pub fn i32(
        env_name: &str,
        default: i32,
    ) -> i32 {
        env::var(env_name).ok().map(|env| env.parse::<i32>().expect("Failed to parse to i32")).unwrap_or(default)
    }

    pub fn u64(
        env_name: &str,
        default: u64,
    ) -> u64 {
        env::var(env_name).ok().map(|env| env.parse::<u64>().expect("Failed to parse to u64")).unwrap_or(default)
    }

    pub fn boolean(
        env_name: &str,
        default: bool,
    ) -> bool {
        env::var(env_name).ok().map(|env| env.parse::<bool>().expect("Failed to parse to bool")).unwrap_or(default)
    }
}
