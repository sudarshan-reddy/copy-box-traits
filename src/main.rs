use reqwest::{Client, Method, RequestBuilder};

trait Authorizer: CloneAuthorizer {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder;
}

impl Clone for Box<dyn Authorizer> {
    fn clone(&self) -> Self {
        self.clone_authorizer()
    }
}

trait CloneAuthorizer {
    fn clone_authorizer(&self) -> Box<dyn Authorizer>;
}

impl<T> CloneAuthorizer for T
where
    T: Authorizer + Clone + 'static,
{
    fn clone_authorizer(&self) -> Box<dyn Authorizer> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct Cl {
    client: Client,
    auth: Box<dyn Authorizer>,
}

impl Cl {
    fn new(auth: Box<dyn Authorizer>) -> Self {
        Cl {
            client: Client::new(),
            auth: auth,
        }
    }

    async fn make_request(&self, url: &str) -> Result<String, Error> {
        let req = self.client.request(Method::GET, url);
        let req = self.auth.authorize_request(req);
        let res = req.send().await?;
        let res_str = res.text().await?;
        Ok(res_str)
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let bearer_client = Cl::new(Box::new(BearerTokenAuthorizer::new("key", "value")));
    let resp = bearer_client
        .make_request("http://site-that-needs-bearer-token")
        .await?;
    println!("{}", resp);

    let basic_auth_client = Cl::new(Box::new(BasicAuth::new("user", "password")));
    let resp = basic_auth_client
        .make_request("http://site-that-needs-basic-auth")
        .await?;
    println!("{}", resp);
    Ok(())
}

#[derive(Clone)]
pub struct BasicAuth {
    user: String,
    api_token: String,
}

impl Authorizer for BasicAuth {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder {
        req.basic_auth(self.user.to_owned(), Some(self.api_token.to_owned()))
    }
}

impl BasicAuth {
    fn new(user: &str, value: &str) -> Self {
        BasicAuth {
            user: user.to_string(),
            api_token: value.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct BearerTokenAuthorizer {
    token_key: String,
    token_value: String,
}

impl Authorizer for BearerTokenAuthorizer {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder {
        req.header(reqwest::header::AUTHORIZATION, self.token_value.clone())
    }
}

impl BearerTokenAuthorizer {
    fn new(key: &str, value: &str) -> Self {
        let value = format!("Bearer {}", value.to_string());
        BearerTokenAuthorizer {
            token_key: key.to_string(),
            token_value: value,
        }
    }
}
