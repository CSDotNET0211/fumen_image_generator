mod image;

use std::option::Option;
use reqwest::Client;
use std::env;
use askama::Template;
use axum::{response::Html, routing::get, Router};
use axum::extract::Query;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use dotenv::dotenv;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::image::{create_gif, create_webp};

// テンプレート構造体
#[derive(Template)]
#[template(path = "template.html")]
struct OgpTemplate {
	site_name: String,
	title: String,
	description: String,
	image_url: String,
	redirect_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct GenerateParams {
	data: String,
	style: Option<u8>,
	page: Option<usize>,
	gif: Option<u8>,
}

#[tokio::main]
async fn main() {
	dotenv().unwrap();

	let app = Router::new()
		.route("/ping", get(handler))
		.route("/:filename", get(image_handler))
		.route("/generate", get(ogp_handler));

	let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
		.await
		.unwrap();
	println!("listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
	Html("<h1>Pong!</h1>")
}


async fn ogp_handler(Query(mut params): Query<GenerateParams>) -> Html<String> {
	let query = percent_decode_str(&serde_urlencoded::to_string(&params).unwrap()).decode_utf8().unwrap().to_string();
	let redirect_url;

	if params.data.starts_with("http") {
		redirect_url = params.data.clone();
	} else {
		redirect_url = format!("{}{}", "https://fumen.zui.jp/?", params.data);
	}

	match extract_url(&params.data).await {
		Some(url) => params.data = url,
		_ => {}
	}

	let fumen = fumen::parse(&params.data);

	let page_index = params.page.unwrap_or(0);
	let template = OgpTemplate {
		site_name: format!("{}/{}", page_index + 1, fumen.pages.len()),
		title: "テト譜/Fumen".into(),
		description: fumen.pages[page_index].comment.clone(),
		image_url: format!("{}?{}", env::var("IMAGE_URL").unwrap(), query),
		redirect_url: redirect_url,
	};


	Html(template.render().unwrap())
}

async fn fetch_redirect_url(url: &Url) -> Result<String, reqwest::Error> {
	let client = Client::builder().redirect(reqwest::redirect::Policy::default()).build()?;
	let response = client.get(url.clone()).send().await?;
	let final_url = response.url().clone();


	Ok(final_url.to_string())
}

async fn image_handler(Query(mut params): Query<GenerateParams>) -> impl IntoResponse {
	match extract_url(&params.data).await {
		Some(url) => params.data = url,
		_ => {}
	}
	//let time = Instant::now();

	let fumen = fumen::parse(&params.data);

	let data;
	let header_type;
	if let Some(1) = params.gif {
		if fumen.pages.len() > 30 {
			return (StatusCode::BAD_REQUEST, "too many pages").into_response();
		}

		data = create_gif(&fumen);
		header_type = "image/gif";
	} else {
		data = create_webp(&fumen, params.page.unwrap_or(0));
		header_type = "image/webp";
	}


	let mut headers = axum::http::HeaderMap::new();
	headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(header_type));
	
	(StatusCode::OK, headers, data).into_response()
}

async fn extract_url(url: &str) -> Option<String> {
	if let Ok(url) = Url::parse(url) {
		if let Some(host) = url.host_str() {
			if host == "tinyurl.com" {
				let url = match fetch_redirect_url(&url).await {
					Ok(final_url) => final_url,
					Err(err) => panic!("{}", err),
				};
				return Some(url);
			}
		}
	}

	None
}