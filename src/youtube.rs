use anyhow::Result;
use url::Url;

pub fn is_youtube_video_url<'a>(url: &'a str) -> Result<bool> {
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

    let has_v_query = parsed
        .query_pairs()
        .find(|(name, _)| name == "v")
        .and_then(|_| Some(true))
        .unwrap_or(false);

    return Ok(is_correct_domain && has_watch_path & has_v_query);
}

#[cfg(test)]
mod tests {
    use super::is_youtube_video_url;

    #[test]
    fn it_requires_youtube_host() {
        let url = "https://google.com/watch?v=nrssnHz0Wz8";
        let result = is_youtube_video_url(&url).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn it_works_without_www() {
        let url = "https://youtube.com/watch?v=nrssnHz0Wz8";
        let result = is_youtube_video_url(&url).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn it_works_with_www() {
        let url = "https://www.youtube.com/watch?v=nrssnHz0Wz8";
        let result = is_youtube_video_url(&url).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn it_works_with_mobile() {
        let url = "https://m.youtube.com/watch?v=nrssnHz0Wz8";
        let result = is_youtube_video_url(&url).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn it_works_with_mobile_www() {
        let url = "https://www.m.youtube.com/watch?v=nrssnHz0Wz8";
        let result = is_youtube_video_url(&url).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn it_works_with_music() {
        let url = "https://music.youtube.com/watch?v=nrssnHz0Wz8";
        let result = is_youtube_video_url(&url).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn it_works_with_http() {
        let url = "http://youtube.com/watch?v=nrssnHz0Wz8";
        let result = is_youtube_video_url(&url).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn it_requires_watch() {
        let url = "http://youtube.com/";
        let result = is_youtube_video_url(&url).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn it_requires_v_query() {
        let url = "http://youtube.com/watch?test=test";
        let result = is_youtube_video_url(&url).unwrap();
        assert_eq!(result, false);
    }
}