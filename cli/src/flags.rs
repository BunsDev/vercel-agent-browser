use std::env;

pub struct Flags {
    pub json: bool,
    pub full: bool,
    pub headed: bool,
    pub debug: bool,
    pub session: String,
    pub headers: Option<String>,
}

pub fn parse_flags(args: &[String]) -> Flags {
    let mut flags = Flags {
        json: false,
        full: false,
        headed: false,
        debug: false,
        session: env::var("AGENT_BROWSER_SESSION").unwrap_or_else(|_| "default".to_string()),
        headers: None,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => flags.json = true,
            "--full" | "-f" => flags.full = true,
            "--headed" => flags.headed = true,
            "--debug" => flags.debug = true,
            "--session" => {
                if let Some(s) = args.get(i + 1) {
                    flags.session = s.clone();
                    i += 1;
                }
            }
            "--headers" => {
                if let Some(h) = args.get(i + 1) {
                    flags.headers = Some(h.clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    flags
}

pub fn clean_args(args: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let mut skip_next = false;

    // Global flags that should be stripped from command args
    const GLOBAL_FLAGS: &[&str] = &["--json", "--full", "--headed", "--debug"];
    // Global flags that take a value (need to skip the next arg too)
    const GLOBAL_FLAGS_WITH_VALUE: &[&str] = &["--session", "--headers"];

    for arg in args.iter() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if GLOBAL_FLAGS_WITH_VALUE.contains(&arg.as_str()) {
            skip_next = true;
            continue;
        }
        // Only strip known global flags, not command-specific flags
        if GLOBAL_FLAGS.contains(&arg.as_str()) || arg == "-f" {
            continue;
        }
        result.push(arg.clone());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn test_parse_headers_flag() {
        let flags = parse_flags(&args(r#"open example.com --headers {"Auth":"token"}"#));
        assert_eq!(flags.headers, Some(r#"{"Auth":"token"}"#.to_string()));
    }

    #[test]
    fn test_parse_headers_flag_with_spaces() {
        // Headers JSON is passed as a single quoted argument in shell
        let input: Vec<String> = vec![
            "open".to_string(),
            "example.com".to_string(),
            "--headers".to_string(),
            r#"{"Authorization": "Bearer token"}"#.to_string(),
        ];
        let flags = parse_flags(&input);
        assert_eq!(flags.headers, Some(r#"{"Authorization": "Bearer token"}"#.to_string()));
    }

    #[test]
    fn test_parse_no_headers_flag() {
        let flags = parse_flags(&args("open example.com"));
        assert!(flags.headers.is_none());
    }

    #[test]
    fn test_clean_args_removes_headers() {
        let input: Vec<String> = vec![
            "open".to_string(),
            "example.com".to_string(),
            "--headers".to_string(),
            r#"{"Auth":"token"}"#.to_string(),
        ];
        let clean = clean_args(&input);
        assert_eq!(clean, vec!["open", "example.com"]);
    }

    #[test]
    fn test_clean_args_removes_headers_at_start() {
        let input: Vec<String> = vec![
            "--headers".to_string(),
            r#"{"Auth":"token"}"#.to_string(),
            "open".to_string(),
            "example.com".to_string(),
        ];
        let clean = clean_args(&input);
        assert_eq!(clean, vec!["open", "example.com"]);
    }

    #[test]
    fn test_headers_with_other_flags() {
        let input: Vec<String> = vec![
            "open".to_string(),
            "example.com".to_string(),
            "--headers".to_string(),
            r#"{"Auth":"token"}"#.to_string(),
            "--json".to_string(),
            "--headed".to_string(),
        ];
        let flags = parse_flags(&input);
        assert_eq!(flags.headers, Some(r#"{"Auth":"token"}"#.to_string()));
        assert!(flags.json);
        assert!(flags.headed);
        
        let clean = clean_args(&input);
        assert_eq!(clean, vec!["open", "example.com"]);
    }
}
