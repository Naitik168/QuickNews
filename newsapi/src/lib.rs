#[cfg(feature = "async")]
use reqwest::Method;
use url::Url;
use serde::Deserialize;

const BASE_URL : &str =  "https://newsapi.org/v2";


//enum for api error types
#[derive(thiserror::Error, Debug)]
pub enum NewsApiError {
    #[error("Failed fetching articles")]
    RequestFailed(#[from] ureq::Error),
    #[error("Failed converting response to string")]
    ResponseToStringFailed(#[from] std::io::Error),
    #[error("Article Parsing failed")]
    ArticleParseFailed(serde_json::Error),
    #[error("Url Parsing failed")]
    UrlParseingFailed(#[from] url::ParseError),
    #[error("Request failed: {0}")]
    BadRequest(&'static str),
    #[error("Async Request failed")]
    #[cfg(feature = "async")]
    AsyncRequestFailed(#[from] reqwest::Error),

}

//structure to store response from API (vector of articles)
#[derive(Deserialize, Debug)]
pub struct NewsApiResponse {
    status : String, 
    pub articles: Vec<Article>,
    code : Option<String>
}

impl NewsApiResponse {
    pub fn articles(&self) -> &Vec<Article>{
        &self.articles
    }
}

//structure of each article
#[derive(Deserialize, Debug)]
pub struct Article {
    title: String,
    url: String,
    description:Option<String>
}

impl Article {

    pub fn title(&self) -> &str {
        &self.title
    }
    
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn desc(&self) -> Option<&String> {
        self.description.as_ref()
    }
    
}

//enum types for setting endpoint for API response
pub enum Endpoint {
    TopHeadlines,
}

impl ToString for Endpoint{
    fn to_string(&self) -> String {
     match self 
     {
        Self::TopHeadlines => "top-headlines".to_string()
     }   
    }
}

//enum types for setting country for API response 
pub enum Country {
    In,
}

impl ToString for Country{
    fn to_string(&self) -> String {
     match self {
        Self::In => "in".to_string()
     }   
    }
}

//structure of news api (to build url)
pub struct NewsAPI {
    api_key : String,
    endpoint : Endpoint,
    country : Country,
}

impl NewsAPI {
    pub fn new(api_key : &str) -> NewsAPI {
        NewsAPI {
            api_key : api_key.to_string(),
            endpoint : Endpoint::TopHeadlines,
            country : Country::In,
        }
    }

    pub fn endpoint(&mut self, endpoint:Endpoint) -> &mut NewsAPI{
        self.endpoint = endpoint;
        self
    }

    pub fn country(&mut self, country:Country) -> &mut NewsAPI{
        self.country = country;
        self
    }

    //preparing url for api
    fn create_url(&self) -> Result<String, NewsApiError> {
        let mut url = Url::parse(BASE_URL)?;
        url.path_segments_mut().unwrap().push(&self.endpoint.to_string());
        
        let country = format!("country={}", self.country.to_string());
        url.set_query(Some(&country));
    
        Ok(url.to_string())   
    }

    //fetching data from api
    pub fn fetch(&self) -> Result<NewsApiResponse, NewsApiError> {
        let url = self.create_url()?;
        let req = ureq::get(&url).set("Authorization", &self.api_key);
        let response : NewsApiResponse = req.call()?.into_json()?;

        match response.status.as_str() {
            "ok" => return Ok(response),
            _ => return Err(map_parsing_err(response.code))
        }
    }

    //implementation of asynchronization, allows program to work further while waiting for api response
    #[cfg(feature = "async")]
    pub async fn fetch_async(&self) -> Result<NewsApiResponse,NewsApiError> {
        let url = self.create_url()?;
        let client = reqwest::Client::new();
        let request = client.request(Method::GET, url).header("Authorization", &self.api_key).build().map_err(|e| NewsApiError::AsyncRequestFailed(e))?;
        
        let response : NewsApiResponse = client.execute(request).await?.json().await.map_err(|e|NewsApiError::AsyncRequestFailed(e))?;
    
        match response.status.as_str() {
            "ok" => return Ok(response),
            _ => return Err(map_parsing_err(response.code))
        }
    }

}

//for mapping error during api procedures
fn map_parsing_err(code : Option<String>) -> NewsApiError {
    if let Some(code) = code {
        match code.as_str() {
            "apiKeyDisable" => NewsApiError::BadRequest("Your API key has been disables"),
            _ => NewsApiError::BadRequest("Unknown error"),
        }
    }else {
        NewsApiError::BadRequest("Unknown error")
    }
}