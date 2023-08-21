## 私人图床搭建
更多教程，请查看我的博客，不定期更新分享很多cool的东西，[直达链接](https://blog.coderluny.com/)
### 1、目标:

- **安全性**：使用HTTPS进行加密。
- **私密性**：完全自我管理的存储。
- **独创性**：尽可能自行编写代码。





### 2、整体方案

**上传流程**：

```mermaid
graph LR
  A[upic] -->|https| B[vps]
  B -->|frp| C[PC]
```

**显示流程**：

```mermaid
graph LR
  C[PC] -->|frp| B[vps]
  B[vps] -->|https| A[web]
```

PC上运行一个图床web服务器，提供上传和下载功能。



### 3、图床服务器源码

```rust
use axum::{
    extract::{Multipart, Path},
    routing::get,
    routing::post,
    Json, Router,
};
use hyper::{Body, Response, StatusCode};
// use multipart::server::Multipart;
use std::{
    net::SocketAddr,
    path::{self},
};
use tokio::fs;
// use tokio_stream::wrappers::BytesStream;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

async fn serve_image(Path(filename): Path<String>) -> Result<Response<Body>, String> {
    if filename.find("..").is_some() || filename.find("/").is_some() {
        return Ok(hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()
            .into());
    }
    if filename.find(".png").is_none() {
        return Ok(hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()
            .into());
    }
    let path = format!("uploads/{}", filename);
    if let Ok(data) = fs::read(&path).await {
        Ok(hyper::Response::new(Body::from(data)))
    } else {
        Ok(hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()
            .into())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct UploadResp {
    dst: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct UploadReq {
    file: String,
}

async fn upload(data: String) -> Result<Json<UploadResp>, StatusCode> {
    let data: UploadReq = serde_json::from_str(&data).map_err(|_| StatusCode::BAD_REQUEST)?;
    let data = base64::decode(data.file).map_err(|_| StatusCode::BAD_REQUEST)?;
    let base_path = std::path::Path::new("uploads");
    if !base_path.exists() {
        fs::create_dir(base_path).await.unwrap();
    }
    let file_name = format!("{}.png", Uuid::new_v4());
    // let filename = format!("uploads/1.png");
    fs::write(&base_path.join(&file_name), data).await.unwrap();

    Ok(Json(UploadResp {
        dst: format!(
            "https://img.example.com:444/{}", // 主意!!!!这儿修改为你的服务器域名和端口
            base_path.join(&file_name).to_string_lossy()
        ),
    }))
}

#[tokio::main]
async fn main() {
    // std::env::set_var("RUST_LOG", "trace");
    env_logger::init();
    let app = Router::new()
        .route("/upload", post(upload))
        .route("/uploads/:filename", get(serve_image));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

```

    /upload：使用POST方法，供upic上传图片。
    /uploads：使用GET方法，获取图片数据。



### 4、部署

#### 4.1 在vps上

设置frp实现内网穿透，

##### a、下载frp

```bash
wget https://github.com/fatedier/frp/releases/download/v0.51.3/frp_0.51.3_linux_arm64.tar.gz
tar -xvzf ./frp_0.51.3_linux_arm64.tar.gz
```

##### b、修改配置文件

```bash
vim frps.ini
```

配置为:

```ini
[common]
bind_port = 7000
# 密码，修改为自己的
token = 123456
# https端口
vhost_https_port = 444
```

##### c、后台运行frp服务端

```bash
nohup ./frps -c frps.ini &
```



#### 4.2 PC上

##### a、下载frp

```bash
wget https://github.com/fatedier/frp/releases/download/v0.51.3/frp_0.51.3_linux_arm64.tar.gz
tar -xvzf ./frp_0.51.3_linux_arm64.tar.gz
```

##### b、修改客户端配置文件

```bash
vim frpc.ini
```

配置为:

```ini
[common]
# vps 公网ip
server_addr = 1.1.1.1 
server_port = 7000
# 密码
token = 123456
# 端口，与服务器保持一致
vhost_https_port = 444

[ssh]
type = tcp
local_ip = 127.0.0.1
local_port = 22
remote_port = 6000
 
[test_https2http]
type = https
# 域名
custom_domains = img.example.com
plugin = https2http
# 图床服务器的端口，如果没修改源码，默认不用改
plugin_local_addr = 127.0.0.1:3000
# https证书，自行搜索如何获取https证书
plugin_crt_path = ./server.crt
plugin_key_path = ./server.key
plugin_host_header_rewrite = 127.0.0.1
plugin_header_X-From-Where = frp
```

##### c、后台运行frp客户端

```bash
nohup ./frpc -c frpc.ini &
```

##### d、运行图床http服务器

```bash
nohup ./img_server &
```



####  4.4 upic设置

添加一个自定义的图床：

![upic设置](https://img.coderluny.com:444/uploads/85993979-a1d1-4bf6-a316-e57996724303.png)

只需修改域名部分。完成设置后，可以进行验证，如果一切正常，将返回一个URL链接。


