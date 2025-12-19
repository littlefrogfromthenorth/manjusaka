
use super::{ApiResponse,JsonResponse};
use crate::models;
use crate::core::AppState;

use poem::{
    session::Session, 
    web::{Data,Redirect},
    http::StatusCode,
    Error,
    Result
};

use poem_openapi::{
    OpenApi, 
    payload::{Response,Json,Binary}
};
use sea_orm::{DatabaseConnection,EntityTrait,ColumnTrait,QueryFilter};
use captcha_rs::CaptchaBuilder;


pub mod types {
    use poem_openapi::Object;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Object, Deserialize, Serialize, Default, Clone)]
    pub struct UserInfo { //根据前端vbenadmin框架构造出所需要的字段
        pub userid: String,
        pub username: String,
        pub token: String,
        pub role: String,
        pub target: String,
    }

    #[derive(Debug, Object)]
    pub struct LoginReq {
        pub username: String,
        pub password: String,
        pub captcha: String,
        pub newpass: Option<String>,
    }

}


pub struct UserApi;

#[OpenApi]
impl UserApi {

    #[oai(path = "/captcha", method = "get")]
    pub async fn captcha(
        &self,
        sess: &Session
    ) -> Result<Response<Binary<Vec<u8>>>> {

        let captcha = CaptchaBuilder::new()
            .length(4)
            .width(120)
            .height(50)
            .dark_mode(false)
            .complexity(3) // min: 1, max: 10
            .compression(20) // min: 1, max: 99
            .build();

        sess.set("VCODE",&captcha.text.to_uppercase());

        let mut imagebytes = Vec::new();
        let _ = captcha.image.write_to(
            &mut std::io::Cursor::new(&mut imagebytes), 
            image::ImageFormat::Png); 

        Ok(Response::new(Binary(imagebytes)).header("Content-Type","image/png"))

    }

    #[oai(path = "/captchabase64", method = "get")]
    pub async fn captchabase64(
        &self,
        sess: &Session
    ) -> JsonResponse<String> {

        let captcha = CaptchaBuilder::new()
            .length(4)
            .width(120)
            .height(50)
            .dark_mode(false)
            .complexity(3) // min: 1, max: 10
            .compression(20) // min: 1, max: 99
            .build();

        sess.set("VCODE",&captcha.text.to_uppercase());

        let mut response = ApiResponse::default();
        response.with_data(captcha.to_base64());

        Json(response)
    }


    #[oai(path = "/login", method = "post")]
    pub async fn login(
        &self,
        sess: &Session,
        app: Data<&AppState>,
        Json(req): Json<types::LoginReq>
    ) -> JsonResponse<types::UserInfo> {

        let mut response = ApiResponse::default();

        let vcode = sess.get::<String>("VCODE").unwrap_or_default();
        sess.remove("VCODE");

        if !req.captcha.is_empty() && !vcode.is_empty() && req.captcha.to_uppercase() == vcode {
            if let Ok(Some(user)) = models::prelude::Users::find()
                .filter(models::users::Column::Username.eq(&req.username))
                .one(&app.conn).await {
                if user.verify_password(&req.username,&req.password) {
                    let userinfo = types::UserInfo{
                        userid:     user.id,
                        username:   user.username,
                        token:      user.userauth,
                        role:       user.userrole,
                        target:     user.userpid,
                    };

                    sess.set("TARGET",&userinfo.target);
                    sess.set("USER",&userinfo);

                    response.with_data(userinfo).with_code(200);
                }else{
                    response.with_code(500).with_msg("账号或密码错误");
                }
            }else{
                response.with_code(500).with_msg("账号或密码错误");
            }
        }else{
            response.with_code(500).with_msg("验证码不正确");
        }
        Json(response)
    }

    #[oai(path = "/password", method = "post")]
    pub async fn password(
        &self,
        sess: &Session,
        app: Data<&AppState>,
        Json(req): Json<types::LoginReq>
    ) -> JsonResponse<String> {

        let mut response = ApiResponse::default();

        let vcode = sess.get::<String>("VCODE").unwrap_or_default();
        sess.remove("VCODE");

        if !req.captcha.is_empty() && !vcode.is_empty() && req.captcha.to_uppercase() == vcode {
            let userinfo = match sess.get::<types::UserInfo>("USER") {
                Some(info) => info,
                None => {
                    response.with_code(401).with_msg("请先登录");
                    return Json(response);
                }
            };

            if let Ok(Some(user)) = models::prelude::Users::find()
                .filter(models::users::Column::Username.eq(&userinfo.username))
                .one(&app.conn).await{
                if user.verify_password(&userinfo.username,&req.password) {
                    if let Some(newpass) = req.newpass{
                        if let Err(e) = user.update_password(&app.conn, &userinfo.username,&newpass).await{
                            response.with_code(500).with_msg(format!("{:?}",e));
                        }else{
                            response.with_code(200).with_msg("ok");
                        }
                    }else{
                        response.with_code(400).with_msg("新密码不能为空");
                    }
                }else{
                    response.with_code(401).with_msg("原密码不正确");
                }
            }else{ 
                response.with_code(500).with_msg("用户不存在");
            }
        }else{
            response.with_code(500).with_msg("验证码不正确");
        }
        Json(response)
    }
}


