// 回归守卫:capabilities/default.json 中 http 插件 scope 的 URL 模式,
// 必须能匹配「带显式端口的局域网 HTTP 服务」和「标准 HTTPS API」。
// 解析逻辑复刻 tauri-plugin-http 的 scope.rs(URLPattern 语义,非 glob;
// 端口未写 * 时只匹配默认端口,这是本次修复的坑)。
use serde_json::Value;
use urlpattern::{UrlPattern, UrlPatternInit, UrlPatternMatchInput};

fn parse_pattern(s: &str) -> UrlPattern {
    let mut init = urlpattern::UrlPatternInit::parse_constructor_string::<regex::Regex>(s, None)
        .expect("无效的 URL pattern");
    if init.search.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
        init.search.replace("*".to_string());
    }
    if init.hash.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
        init.hash.replace("*".to_string());
    }
    if init
        .pathname
        .as_ref()
        .map(|p| p.is_empty() || p == "/")
        .unwrap_or(true)
    {
        init.pathname.replace("*".to_string());
    }
    UrlPattern::parse(init, Default::default()).expect("URL pattern 编译失败")
}

fn is_allowed(patterns: &[String], url: &str) -> bool {
    let url = url::Url::parse(url).unwrap();
    patterns.iter().any(|p| {
        parse_pattern(p)
            .test(UrlPatternMatchInput::Url(url.clone()))
            .unwrap_or(false)
    })
}

#[test]
fn http_scope_allows_custom_ai_endpoints() {
    let caps: Value = serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
    let patterns: Vec<String> = caps["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["identifier"] == "http:default")
        .flat_map(|p| {
            p["allow"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|e| e["url"].as_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(!patterns.is_empty(), "http:default 缺少 allow scope");

    // 带显式端口的局域网 OpenAI 兼容服务(本次 bug 现场)
    assert!(is_allowed(&patterns, "http://192.168.3.3:8084/v1/chat/completions"));
    // 标准 HTTPS API
    assert!(is_allowed(&patterns, "https://api.openai.com/v1/chat/completions"));
    // 本地部署服务
    assert!(is_allowed(&patterns, "http://localhost:11434/v1/chat/completions"));
}
