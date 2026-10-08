use anyhow::{Result, anyhow, bail};
use base64::{Engine as _, engine::general_purpose};
use serde_json::{Value, json};
use serde_yaml_ng::Mapping;
use std::collections::{HashMap, HashSet};
use tauri::Url;

const SELECT_GROUP: &str = "Blancwatch";
const FALLBACK_GROUP: &str = "Blancwatch Auto";

pub(super) fn decode_base64(data: &str) -> Option<String> {
    let compact: String = data.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    [
        general_purpose::STANDARD,
        general_purpose::STANDARD_NO_PAD,
        general_purpose::URL_SAFE,
        general_purpose::URL_SAFE_NO_PAD,
    ]
    .iter()
    .find_map(|engine| engine.decode(&compact).ok())
    .and_then(|bytes| String::from_utf8(bytes).ok())
}

pub(super) fn profile_title(raw: &str) -> Option<String> {
    let title = raw.trim();
    let title = match title.strip_prefix("base64:") {
        Some(encoded) => decode_base64(encoded)?,
        None => title.to_owned(),
    };
    let title = title.trim();
    (!title.is_empty() && !title.chars().any(char::is_control)).then(|| title.to_owned())
}

pub(super) fn adapt_remote_profile(data: &str) -> Result<String> {
    let data = data.trim_start_matches('\u{feff}');
    if let Ok(yaml) = serde_yaml_ng::from_str::<Mapping>(data) {
        if yaml.contains_key("proxies") || yaml.contains_key("proxy-providers") {
            return Ok(data.to_owned());
        }
        bail!("profile does not contain `proxies` or `proxy-providers`");
    }

    let decoded;
    let feed = if data.trim_start().starts_with("vless://") {
        data
    } else {
        decoded = decode_base64(data).ok_or_else(|| anyhow!("remote profile is not Clash YAML or a VLESS feed"))?;
        &decoded
    };
    let mut proxies = Vec::new();
    let mut names = HashSet::from([
        SELECT_GROUP.to_owned(),
        FALLBACK_GROUP.to_owned(),
        "DIRECT".to_owned(),
        "REJECT".to_owned(),
        "GLOBAL".to_owned(),
        "COMPATIBLE".to_owned(),
    ]);
    for (index, line) in feed.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut proxy = parse_vless(line).map_err(|error| anyhow!("VLESS feed line {}: {error}", index + 1))?;
        let original = proxy["name"]
            .as_str()
            .ok_or_else(|| anyhow!("VLESS node has no name"))?
            .to_owned();
        let mut name = original.clone();
        let mut suffix = 2;
        while !names.insert(name.clone()) {
            name = format!("{original} ({suffix})");
            suffix += 1;
        }
        proxy["name"] = name.into();
        proxies.push(proxy);
    }
    if proxies.is_empty() {
        bail!("VLESS feed contains no nodes");
    }
    let nodes: Vec<_> = proxies.iter().map(|proxy| proxy["name"].clone()).collect();
    let mut choices = vec![Value::from(FALLBACK_GROUP)];
    choices.extend(nodes.iter().cloned());
    let config = json!({
        "proxies": proxies,
        "proxy-groups": [
            {"name": SELECT_GROUP, "type": "select", "proxies": choices},
            {
                "name": FALLBACK_GROUP, "type": "fallback", "proxies": nodes,
                "url": "https://www.gstatic.com/generate_204", "interval": 300,
                "timeout": 5000, "max-failed-times": 3, "lazy": true
            }
        ],
        "rules": [format!("MATCH,{SELECT_GROUP}")]
    });
    serde_yaml_ng::to_string(&config).map_err(|_| anyhow!("failed to encode VLESS profile"))
}

fn decode_component(raw: &str) -> Result<String> {
    let bytes = raw.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'%'
            && !bytes
                .get(index + 1..index + 3)
                .is_some_and(|pair| pair.iter().all(u8::is_ascii_hexdigit))
        {
            bail!("invalid URI encoding");
        }
    }
    percent_encoding::percent_decode_str(raw)
        .decode_utf8()
        .map(|value| value.into_owned())
        .map_err(|_| anyhow!("invalid URI encoding"))
}

fn parse_params(url: &Url) -> Result<HashMap<String, String>> {
    let mut params = HashMap::new();
    for pair in url
        .query()
        .unwrap_or_default()
        .split('&')
        .filter(|pair| !pair.is_empty())
    {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = decode_component(key)?;
        if !matches!(
            key.as_str(),
            "type"
                | "security"
                | "encryption"
                | "flow"
                | "fp"
                | "sni"
                | "pbk"
                | "sid"
                | "host"
                | "path"
                | "alpn"
                | "headerType"
        ) {
            bail!("unsupported VLESS parameter");
        }
        if params
            .insert(key, decode_component(&value.replace('+', " "))?)
            .is_some()
        {
            bail!("duplicate VLESS parameter");
        }
    }
    Ok(params)
}

fn parse_vless(line: &str) -> Result<Value> {
    if !line.starts_with("vless://") {
        bail!("unsupported protocol");
    }
    let url = Url::parse(line).map_err(|_| anyhow!("invalid VLESS URI"))?;
    let server = url.host_str().ok_or_else(|| anyhow!("missing server"))?;
    let port = url
        .port()
        .filter(|port| *port != 0)
        .ok_or_else(|| anyhow!("invalid port"))?;
    let uuid = decode_component(url.username())?;
    if url.password().is_some()
        || !matches!(url.path(), "" | "/")
        || uuid.len() != 36
        || !uuid.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
    {
        bail!("invalid VLESS identity or authority");
    }
    let params = parse_params(&url)?;
    let value = |key| params.get(key).map(String::as_str).unwrap_or_default();
    let network = match value("type") {
        "" | "tcp" => "tcp",
        "ws" | "websocket" => "ws",
        _ => bail!("unsupported VLESS transport"),
    };
    let security = value("security");
    if !matches!((network, security), ("tcp", "reality") | ("ws", "tls")) {
        bail!("unsupported VLESS transport/security combination");
    }
    if !matches!(value("encryption"), "" | "none") || !matches!(value("headerType"), "" | "none") {
        bail!("unsupported VLESS encryption or header type");
    }
    let flow = value("flow");
    if !matches!(flow, "" | "xtls-rprx-vision") || (network == "ws" && !flow.is_empty()) {
        bail!("unsupported VLESS flow");
    }
    if !matches!(
        value("fp"),
        "" | "firefox" | "chrome" | "safari" | "ios" | "android" | "edge" | "360" | "qq" | "random" | "randomized"
    ) {
        bail!("unsupported TLS fingerprint");
    }
    let name = url
        .fragment()
        .map(decode_component)
        .transpose()?
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| format!("VLESS {server}:{port}"));
    let mut proxy = json!({
        "name": name, "type": "vless", "server": server.trim_matches(['[', ']']),
        "port": port, "uuid": uuid, "udp": true, "tls": true, "network": network
    });
    for (key, target) in [("sni", "servername"), ("fp", "client-fingerprint"), ("flow", "flow")] {
        if !value(key).is_empty() {
            proxy[target] = value(key).into();
        }
    }
    if !value("alpn").is_empty() {
        proxy["alpn"] = value("alpn").split(',').map(Value::from).collect();
    }
    let (key, options) = transport_options(&params)?;
    proxy[key] = options;
    Ok(proxy)
}

fn transport_options(params: &HashMap<String, String>) -> Result<(&'static str, Value)> {
    let value = |key| params.get(key).map(String::as_str).unwrap_or_default();
    if value("security") == "reality" {
        if !value("host").is_empty() || !value("path").is_empty() {
            bail!("WebSocket fields on a TCP node");
        }
        let public_key = value("pbk");
        let short_id = value("sid");
        if general_purpose::URL_SAFE_NO_PAD
            .decode(public_key)
            .ok()
            .is_none_or(|bytes| bytes.len() != 32)
            || value("sni").is_empty()
            || short_id.len() > 16
            || !short_id.len().is_multiple_of(2)
            || !short_id.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            bail!("invalid REALITY settings");
        }
        Ok(("reality-opts", json!({"public-key": public_key, "short-id": short_id})))
    } else {
        if params.contains_key("pbk") || params.contains_key("sid") {
            bail!("REALITY fields on a TLS node");
        }
        let mut ws = json!({});
        if params.contains_key("host") {
            ws["headers"] = json!({"Host": value("host")});
        }
        if params.contains_key("path") {
            ws["path"] = value("path").into();
        }
        Ok(("ws-opts", ws))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context as _;

    const REALITY: &str = "vless://00000000-0000-4000-8000-000000000001@tcp.example.test:443?type=tcp&security=reality&encryption=none&flow=xtls-rprx-vision&fp=firefox&sni=tls.example.test&pbk=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&sid=0123abcd#Example%20TCP";
    const WS: &str = "vless://00000000-0000-4000-8000-000000000002@ws.example.test:443?type=ws&security=tls&encryption=none&fp=firefox&sni=sni.example.test&host=host.example.test&path=%2Fsocket%3Fed%3D2048#Example%20WS";

    #[test]
    fn raw_and_base64_feeds_preserve_transports_and_route_to_vpn() -> Result<()> {
        let feed = format!("{}{}", format!("{REALITY}\n").repeat(56), format!("{WS}\n").repeat(6));
        let raw = adapt_remote_profile(&feed)?;
        assert_eq!(raw, adapt_remote_profile(&general_purpose::STANDARD.encode(feed))?);
        let yaml: serde_yaml_ng::Value = serde_yaml_ng::from_str(&raw)?;
        let tcp = &yaml["proxies"][0];
        assert_eq!(tcp["network"], "tcp");
        assert_eq!(tcp["flow"], "xtls-rprx-vision");
        assert_eq!(tcp["client-fingerprint"], "firefox");
        assert_eq!(tcp["servername"], "tls.example.test");
        assert_eq!(
            tcp["reality-opts"]["public-key"],
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );
        assert_eq!(tcp["reality-opts"]["short-id"], "0123abcd");
        assert_eq!(yaml["proxies"].as_sequence().map(Vec::len), Some(62));
        assert_eq!(yaml["proxies"][1]["name"], "Example TCP (2)");
        let ws = &yaml["proxies"][56];
        assert_eq!(ws["network"], "ws");
        assert_eq!(ws["servername"], "sni.example.test");
        assert_eq!(ws["ws-opts"]["headers"]["Host"], "host.example.test");
        assert_eq!(ws["ws-opts"]["path"], "/socket?ed=2048");
        assert_eq!(ws["client-fingerprint"], "firefox");
        assert_eq!(ws["tls"], true);
        assert!(
            ws["ws-opts"]
                .as_mapping()
                .is_some_and(|opts| !opts.contains_key("v2ray-http-upgrade"))
        );
        let groups = &yaml["proxy-groups"];
        assert_eq!(groups[0]["proxies"][0], FALLBACK_GROUP);
        assert_eq!(groups[1]["type"], "fallback");
        assert_eq!(groups[1]["timeout"], 5000);
        assert_eq!(groups[1]["interval"], 300);
        assert_eq!(groups[1]["proxies"].as_sequence().map(Vec::len), Some(62));
        assert_eq!(yaml["rules"][0], format!("MATCH,{SELECT_GROUP}"));
        Ok(())
    }

    #[test]
    fn clash_yaml_is_unchanged_and_unsupported_feed_is_rejected_without_values() -> Result<()> {
        let yaml = "# Existing YAML\nproxies: []\nproxy-groups: []\nrules: [MATCH,DIRECT]\n";
        assert_eq!(adapt_remote_profile(yaml)?, yaml);
        assert_eq!(adapt_remote_profile("proxy-providers: {}\n")?, "proxy-providers: {}\n");
        for invalid in [
            WS.replace("type=ws", "type=xhttp"),
            format!("{WS}\n{}", REALITY.replace("?type=", "?unknown=private-value&type=")),
            format!("{WS}\nvless://malformed"),
        ] {
            let error = adapt_remote_profile(&invalid)
                .err()
                .context("unsupported feed must fail")?
                .to_string();
            assert!(!error.contains("private-value"));
            assert!(!error.contains("example.test"));
            assert!(!error.contains("00000000"));
        }
        assert_eq!(
            profile_title("base64:RXhhbXBsZSBQcm9maWxl"),
            Some("Example Profile".to_owned())
        );
        Ok(())
    }
}
