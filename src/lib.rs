use worker::*;

mod services;
mod utils;
mod endpoints;
mod models;


fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> worker::Result<Response> {
    log_request(&req);
    utils::set_panic_hook();

    // extract api key
    let api_key = extract_api_key(&env)?;

    let router = Router::new();
    let auth_res = is_authed(req.headers(), api_key);
        
    if let Err(err) = auth_res {
        return err;
    };

    router
        .post_async("/api/v1/images/overlays", |_req, _ctx| async move {
            endpoints::images::ep_add_image_overlay(_req).await
        })
        .run(req, env)
        .await
}

fn extract_api_key(env: &Env) -> std::result::Result<String, worker::Error> {
    let api_key = env.secret("WORKER_API_KEY");

    match api_key {
        Ok(key) => return Ok(key.to_string()),
        Err(_) => {
            console_log!("missing WORKER_API_KEY in wrangler.toml");
            return Err(worker::Error::RustError("An error occured".to_string()))
        }
    }
}

fn is_authed(headers: &Headers, api_key: String) -> std::result::Result<(), worker::Result<Response>> {
    let auth_header_val = headers.get("Authorization");

    match auth_header_val {
        Ok(opt) => match opt {
            Some(header_val) => {
                if header_val == api_key {
                    return Ok(())
                }
                return Err(Response::error("You are not authorized to access this resource".to_string(), 401))
            },
            None => return Err(Response::error("You are not authorized to access this resource".to_string(), 401))
        },
        Err(_) => return Err(Response::error("You are not authorized to access this resource".to_string(), 401))
    }
}
