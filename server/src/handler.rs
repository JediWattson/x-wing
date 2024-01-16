use std::{convert::Infallible, fs};
use hyper::{body::{Bytes, Frame, Incoming}, Request, Response};
use http_body_util::{Full, BodyExt};
use askama::Template;

#[derive(Template)]
#[template(path = "pages/index.html")]
struct Home {}

#[derive(Template)]
#[template(path = "pages/404.html")]
struct NotFound {}

fn render_route(path: &str) -> Full<Bytes> {
    let page = match path {
        "/" => { Home {}.render().unwrap() },
        _ => { NotFound {}.render().unwrap() },
    };
    Full::new(page.into())
}

fn get_bin(bin: &str) -> Full<Bytes> {
    let path = format!("web/{}", bin);
    fs::read(path).unwrap().into()
}

fn get_file_string(dir: &str, module: &str) -> Full<Bytes> {
    let path = format!("{}/{}", dir, module);
    fs::read_to_string(path).unwrap().into()
}

async fn get_content_type(req: Request<Incoming>) -> (String, Full<Bytes>) {
    let path = req.uri().path().to_owned();
    let extension = path.split('.').last().unwrap();    
    if path == "/video_upload" {
        let body = req.into_body().collect().await.unwrap();
        let collect = body.collect().await.unwrap();
        let data = String::from_utf8(collect.to_bytes().to_vec()).unwrap();
        return (String::from("application/json"), Full::new(data.into()));
    }

    let contents = match extension {
        "css" => { get_file_string("server/src/public", &path) },
        "js" => { get_file_string("web", &path) },
        "wasm" => { get_bin(&path) },
        _ => { render_route(&path) },
    };

    let contens_type = match extension {
        "css" => { "text/css" },
        "js" => { "application/javascript" },
        "wasm" => { "application/wasm" },
        _ => { "text/html" },
    };
    (String::from(contens_type), contents)
}

pub async fn on_req(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let (contens_type, contents) = get_content_type(req).await;
    
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", contens_type)
        .body(contents)
        .unwrap()
    )
}