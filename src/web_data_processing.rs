use once_cell::sync::Lazy;
use regex::Regex;

pub async fn get_links_from_url(url: &str) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
    match reqwest::get(url).await {
        Ok(r) => {
            match r.error_for_status() {
                Ok(r)=>{
                    Ok((
                    get_resource_from_link(r.url().as_str()),
                    match r.text().await {
                        Ok(s) => links_helper(&s),
                        Err(e) => return Err(Box::new(e)),
                    }
                ))
                }
                Err(e)=> Err(Box::new(e)),
            }

        },
        Err(e) => Err(Box::new(e)),
    }
}

pub fn get_resource_from_link(haystack: &str) -> String {
    // println!("{}", haystack);
    static ENDINGS: Lazy<Regex> = Lazy::new(|| Regex::new(r##"\=([^&=]*)\&"##).unwrap());

    match ENDINGS.find(haystack).iter().next(){
        Some(m)=>{m.as_str()[1..m.len()-1].to_string()},
        None => String::new()
    }.replace("%20", " ")

}



pub fn links_helper(haystack: &str) -> Vec<String> {
    // println!("{}", haystack);
    static ENDINGS: Lazy<Regex> = Lazy::new(|| Regex::new(r##"\[\[(.*?)\]\]"##).unwrap());

    ENDINGS
        .find_iter(&haystack)
        .map(|m| {
            m.as_str()
                .split("|")
                .next()
                .unwrap_or_default()
                .to_string()
                .replace("[[", "")
                .replace("]]", "")
        })
        .filter(|s| !s.starts_with("File:"))
        .collect()
}


#[cfg(test)]
mod string_processing_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_links_from_url(){
        let result = get_links_from_url("https://en.wikipedia.org/w/index.php?title=Eustace_Ingram&action=raw");
        let result_ok = result.await.unwrap();
        
        assert_eq!(result_ok,(
            String::from("Eustace_Ingram"),
            Vec::<String>::from([
                "London".to_string()
                , "Henry Willis".to_string()
                , "George Holdich".to_string()
                , "Gray and Davison".to_string()
                , "Category:1839 births".to_string()
                , "Category:1924 deaths".to_string()
                , "Category:British pipe organ builders".to_string()
                , "Category:Music in London".to_string()
        ])));
        
    }

    #[test]
    fn test_get_links_helper(){
        let result = links_helper("[[test]]");
        let expected = Vec::from(["test"]);
        assert!((0..std::cmp::max(result.len(), expected.len())).all(|i| {
            &result[i] == expected[i]
        }));
        
        // multiple 
        let result = links_helper("[[test]] [[ ]]");

        assert_eq!(result, Vec::<String>::from([
            "test".to_string()
           ," ".to_string()
        ]));

        // should not take files, and should not take second half of [[first|second]]
        let result = links_helper("[[test]] [[File:something.jpg]] [[ | ]]");

        assert_eq!(result, Vec::<String>::from([
            "test".to_string()
           ," ".to_string()
        ]));
    }




}
