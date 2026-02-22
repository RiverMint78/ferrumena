pub fn compact_url_for_log(url: &str) -> String {
	if let Some(scheme_pos) = url.find("://") {
		let rest = &url[scheme_pos + 3..];
		if let Some(path_pos) = rest.find('/') {
			return rest[path_pos..].to_string();
		}
		return "/".to_string();
	}

	url.to_string()
}
