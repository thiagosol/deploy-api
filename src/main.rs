use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse, middleware::Logger};
use actix_web_httpauth::extractors::basic::BasicAuth;
use std::process::{Command, Stdio};
use std::{fs::File, io::Read};
use std::env;
use log::info;
use dotenv::dotenv;

#[derive(serde::Deserialize)]
struct DeployRequest {
    service: String,
    branch: Option<String>,
    env_vars: Vec<String>,
}

fn is_authenticated(auth: &BasicAuth) -> bool {
    let user = env::var("DEPLOY_USER").unwrap_or("admin".to_string());
    let pass = env::var("DEPLOY_PASS").unwrap_or("senha123".to_string());

    if let Some(password) = auth.password() {
        return auth.user_id() == user && password == pass;
    }
    false
}

#[post("/deploy")]
async fn deploy(auth: BasicAuth, form: web::Json<DeployRequest>) -> impl Responder {
    if !is_authenticated(&auth) {
        return HttpResponse::Unauthorized().finish();
    }

    let service_name = &form.service;
    let branch = form.branch.as_deref().unwrap_or("main");
    let env_vars = form.env_vars.join(" ");

    let log_file_path = format!("/opt/auto-deploy/{}/deploy.log", service_name);
    
    let mut deploy_cmd = Command::new("/bin/bash");
    deploy_cmd
        .arg("-c")
        .arg(format!(
            "/opt/deploy.sh {} {} {} >> {} 2>&1 &",
            service_name, branch, env_vars, log_file_path
        ))
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    match deploy_cmd.spawn() {
        Ok(_) => HttpResponse::Ok().body(format!("🚀 Deploy iniciado para {}!", service_name)),
        Err(e) => HttpResponse::InternalServerError().body(format!("❌ Erro ao iniciar deploy: {}", e)),
    }
}

#[get("/logs/{service}")]
async fn get_logs(auth: BasicAuth, path: web::Path<String>) -> impl Responder {
    if !is_authenticated(&auth) {
        return HttpResponse::Unauthorized().finish();
    }

    let service_name = path.into_inner();
    let log_file_path = format!("/opt/{}/deploy.log", service_name);

    let mut file = match File::open(&log_file_path) {
        Ok(f) => f,
        Err(_) => return HttpResponse::NotFound().body("Log não encontrado."),
    };

    let mut contents = String::new();
    if let Err(_) = file.read_to_string(&mut contents) {
        return HttpResponse::InternalServerError().body("Erro ao ler log.");
    }

    HttpResponse::Ok().body(contents)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("🚀 Starting API...");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(deploy)
            .service(get_logs)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
