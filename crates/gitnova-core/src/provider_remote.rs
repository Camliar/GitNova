const MAX_SSH_CONFIG_OUTPUT_BYTES: usize = 64 * 1024;

pub fn ssh_alias_and_path(value: &str) -> Option<(&str, &str)> {
    let (alias, path) = if let Some(rest) = value.strip_prefix("ssh://git@") {
        rest.split_once('/')?
    } else {
        let rest = value.strip_prefix("git@")?;
        rest.split_once(':')?
    };
    if !valid_alias(alias) || path.is_empty() || path.contains(['?', '#']) {
        return None;
    }
    Some((alias, path))
}

pub fn configured_hostname(output: &[u8]) -> Option<&str> {
    if output.len() > MAX_SSH_CONFIG_OUTPUT_BYTES {
        return None;
    }
    let value = std::str::from_utf8(output).ok()?;
    let mut hostnames = value.lines().filter_map(|line| {
        let (key, value) = line.split_once(' ')?;
        (key.eq_ignore_ascii_case("hostname") && valid_hostname(value)).then_some(value)
    });
    let hostname = hostnames.next()?;
    if hostnames.next().is_some() {
        return None;
    }
    Some(hostname)
}

pub fn valid_hostname(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && !value.starts_with(['-', '.'])
        && !value.ends_with(['-', '.'])
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
}

fn valid_alias(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_only_safe_ssh_aliases_and_single_hostname() {
        assert_eq!(
            ssh_alias_and_path("git@github-work:owner/repo.git"),
            Some(("github-work", "owner/repo.git"))
        );
        assert_eq!(
            ssh_alias_and_path("ssh://git@gitlab_work/team/repo.git"),
            Some(("gitlab_work", "team/repo.git"))
        );
        assert!(ssh_alias_and_path("git@-oProxyCommand=evil:owner/repo.git").is_none());
        assert!(ssh_alias_and_path("https://github-work/owner/repo.git").is_none());
        assert_eq!(
            configured_hostname(b"host github-work\nhostname github.com\nuser git\n"),
            Some("github.com")
        );
        assert!(configured_hostname(b"hostname github.com\nhostname evil.example\n").is_none());
        assert!(configured_hostname(b"hostname github.com/path\n").is_none());
    }
}
