use anyhow::Result;
use url::Url;

pub fn get_youtube_video_id<'a>(url: &'a str) -> Result<Option<String>> {
    let parsed = Url::parse(url)?;

    let is_correct_domain = match parsed.domain() {
        Some("youtube.com") => true,
        Some("www.youtube.com") => true,
        Some("m.youtube.com") => true,
        Some("www.m.youtube.com") => true,
        Some("music.youtube.com") => true,
        _ => false
    };

    let has_watch_path = parsed
        .path_segments()
        .and_then(|mut segments| segments.next())
        .and_then(|segment| Some(segment == "watch"))
        .unwrap_or(false);

    let youtube_id = parsed
        .query_pairs()
        .find(|(name, _)| name == "v")
        .and_then(|(_, id)| Some(id.to_string()));

    if is_correct_domain && has_watch_path {
        return Ok(youtube_id);
    } else {
        return Ok(None);
    }
}

#[cfg(test)]
mod tests {
    use super::get_youtube_video_id;

    #[test]
    fn it_requires_youtube_host() {
        let url = "https://google.com/watch?v=nrssnHz0Wz8";
        let result = get_youtube_video_id(&url).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn it_works_without_www() {
        let url = "https://youtube.com/watch?v=nrssnHz0Wz8";
        let result = get_youtube_video_id(&url).unwrap();
        assert_eq!(result, Some("nrssnHz0Wz8".to_string()));
    }

    #[test]
    fn it_works_with_www() {
        let url = "https://www.youtube.com/watch?v=nrssnHz0Wz8";
        let result = get_youtube_video_id(&url).unwrap();
        assert_eq!(result, Some("nrssnHz0Wz8".to_string()));
    }

    #[test]
    fn it_works_with_mobile() {
        let url = "https://m.youtube.com/watch?v=nrssnHz0Wz8";
        let result = get_youtube_video_id(&url).unwrap();
        assert_eq!(result, Some("nrssnHz0Wz8".to_string()));
    }

    #[test]
    fn it_works_with_mobile_www() {
        let url = "https://www.m.youtube.com/watch?v=nrssnHz0Wz8";
        let result = get_youtube_video_id(&url).unwrap();
        assert_eq!(result, Some("nrssnHz0Wz8".to_string()));
    }

    #[test]
    fn it_works_with_music() {
        let url = "https://music.youtube.com/watch?v=nrssnHz0Wz8";
        let result = get_youtube_video_id(&url).unwrap();
        assert_eq!(result, Some("nrssnHz0Wz8".to_string()));
    }

    #[test]
    fn it_works_with_http() {
        let url = "http://youtube.com/watch?v=nrssnHz0Wz8";
        let result = get_youtube_video_id(&url).unwrap();
        assert_eq!(result, Some("nrssnHz0Wz8".to_string()));
    }

    #[test]
    fn it_requires_watch() {
        let url = "http://youtube.com/";
        let result = get_youtube_video_id(&url).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn it_requires_v_query() {
        let url = "http://youtube.com/watch?test=test";
        let result = get_youtube_video_id(&url).unwrap();
        assert_eq!(result, None);
    }
}