use actix_cors::Cors;
use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::extractors::basic::BasicAuth;
use dotenv::dotenv;
use log::info;
use std::env;
use std::fs;
use std::process::Command;
use std::{fs::File, io::Read};

const DIR_BASE: &str = "/opt/auto-deploy";

#[derive(serde::Deserialize)]
struct DeployRequest {
    service: String,
    branch: Option<String>,
    env_vars: Vec<String>,
}

fn is_authenticated(auth: &BasicAuth) -> bool {
    let user = env::var("DEPLOY_USER").unwrap();
    let pass = env::var("DEPLOY_PASS").unwrap();

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

    let ssh_key_path = env::var("SSH_PRIVATE_KEY_PATH").unwrap();
    let ssh_user = env::var("SSH_USER").unwrap();
    let ssh_host = env::var("SSH_HOST").unwrap();

    let log_file_path = format!("{}/logs/{}.log", DIR_BASE, service_name);

    if fs::metadata(&log_file_path).is_ok() {
        if let Err(e) = fs::remove_file(&log_file_path) {
            log::warn!("⚠️ Falha ao remover log antigo: {}", e);
        } else {
            log::info!("🗑️ Log antigo removido com sucesso!");
        }
    }

    info!(
        "🚀 Iniciando deploy do serviço: {} na branch: {}",
        service_name, branch
    );

    let output = Command::new("ssh")
        .args(&[
            "-i",
            &ssh_key_path,
            "-o",
            "StrictHostKeyChecking=no",
            &format!("{}@{}", ssh_user, ssh_host),
            &format!(
                "{}/deploy.sh {} {} {} >> {} 2>&1 &",
                DIR_BASE, service_name, branch, env_vars, log_file_path
            ),
        ])
        .output();

    match output {
        Ok(_) => {
            info!("✅ Deploy enviado para execução via SSH");
            HttpResponse::Ok().body(format!("🚀 Deploy iniciado para {}!", service_name))
        }
        Err(e) => {
            log::error!("❌ Erro ao iniciar deploy via SSH: {}", e);
            HttpResponse::InternalServerError().body(format!("❌ Erro ao iniciar deploy: {}", e))
        }
    }
}

#[get("/logs/{service}")]
async fn get_logs(auth: BasicAuth, path: web::Path<String>) -> impl Responder {
    if !is_authenticated(&auth) {
        return HttpResponse::Unauthorized().finish();
    }

    let service_name = path.into_inner();
    let log_file_path = format!("{}/logs/{}.log", DIR_BASE, service_name);

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
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .service(deploy)
            .service(get_logs)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
