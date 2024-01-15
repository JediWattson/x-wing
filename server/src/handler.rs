use std::{convert::Infallible, fs};
use askama::Template;
use hyper::{body::{Bytes, Incoming}, Request, Response};
use http_body_util::Full;

#[derive(Template)]
#[template(path = "pages/index.html")]
struct Home {
    name: String,
}
#[derive(Template)]
#[template(path = "pages/404.html")]
struct NotFound {}

fn render_route(path: &str) -> Full<Bytes> {
    let page = match path {
        "/" => { Home { name: "World".to_string() }.render().unwrap() },
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

fn get_contents(path: &str) -> Full<Bytes> {
    if path.ends_with(".wasm") {
        get_bin(path)
    } else if path.ends_with(".js") {
        get_file_string("web", path)
    } else if path.ends_with(".css") {
        get_file_string("server/src/public", path)
    } else {    
        render_route(path)
    }
}

fn get_content_type(extension: &str) -> String {
    let contens_type = match extension {
        "css" => { "text/css" },
        "js" => { "application/javascript" },
        "wasm" => { "application/wasm" },
        _ => { "text/html" },
    };
    String::from(contens_type)
}

pub async fn on_req(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path();
    let extension = path.split('.').last().unwrap();
    let contens_type = get_content_type(extension);

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", contens_type)
        .body(get_contents(path))
        .unwrap()
    )
}