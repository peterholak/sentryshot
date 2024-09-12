use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use sha1::{Digest, Sha1};
use thiserror::Error;
use url::Url;
use crate::onvif::MonitorOnvifError::{FetchMediaProfilesError, NoMediaProfiles};
use crate::ptz::{MovementKind, PtzCapabilities, PtzDirection};

#[derive(Debug, Error)]
pub enum MonitorOnvifError {
  #[error("onvif fetch media profiles")]
  FetchMediaProfilesError,

  #[error("onvif no media profiles")]
  NoMediaProfiles,
}

pub async fn discover_ptz_capabilities(url: &Url) -> Result<PtzCapabilities, MonitorOnvifError> {
    let (url_without_credentials, security_header) = extract_credentials(url);
    let client = reqwest::Client::new();
    let response = client
        .post(url_without_credentials)
        .body(media_get_profiles_body(&security_header))
        .send()
        .await
        .map_err(|_e| FetchMediaProfilesError)?;
    let body = response.text()
        .await
        .map_err(|_e| FetchMediaProfilesError)?;

    let parsed = parse_ptz_capabilities(&body);
    if parsed.len() > 0 {
        Ok(parsed[0].clone())
    } else {
        Err(NoMediaProfiles)
    }
}

fn extract_credentials(url: &Url) -> (Url, String) {
    let credentials = match (url.username(), url.password()) {
        (username, Some(password)) => Some((username, password)),
        _ => None
    };
    let mut url_without_credentials = url.clone();
    url_without_credentials.set_username("").ok();
    url_without_credentials.set_password(None).ok();

    let security_header = match credentials {
        Some((username, password)) => create_wsse_token(username, password),
        None => "".to_string()
    };

    (url_without_credentials, security_header)
}

pub async fn move_ptz(url: &Url, direction: PtzDirection, capabilities: &PtzCapabilities) {
    let movement_kind = capabilities.preferred_movement(direction);
    match movement_kind {
        Some(MovementKind::Continuous) => {
            move_continuous(url, &capabilities.profile_token, direction).await
        }
        Some(MovementKind::Relative) => {
            move_relative(url, &capabilities.profile_token, direction).await
        }
        Some(MovementKind::Absolute) => {
            move_absolute(url, &capabilities.profile_token, direction).await
        }
        None => {}
    }
}

async fn move_relative(url: &Url, profile_token: &str, direction: PtzDirection) {
    let (url_without_credentials, security_header) = extract_credentials(url);
    let client = reqwest::Client::new();
    // TODO: reuse client?
    let response = client
        .post(url_without_credentials)
        .body(ptz_move_relative_body(profile_token, direction, &security_header))
        .send()
        .await
        .unwrap();
    let _body = response.text()
        .await
        .unwrap();
    // TODO: parse errors, etc.
}

async fn move_absolute(url: &Url, profile_token: &str, direction: PtzDirection) {
    todo!("will need to split into 2 requests")
}

async fn move_continuous(url: &Url, profile_token: &str, direction: PtzDirection) {
    // TODO: reuse client over a longer time
    let client = reqwest::Client::new();
    move_continuous_start(url, profile_token, direction, &client).await;
    // TODO: estimate latency and adjust the stop wait
    let duration_ms = if direction.is_zoom() { 1000 } else { 500 };
    tokio::time::sleep(std::time::Duration::from_millis(duration_ms)).await;
    move_stop(url, profile_token, &client).await;
}

async fn move_continuous_start(url: &Url, profile_token: &str, direction: PtzDirection, client: &reqwest::Client) {
    let (url_without_credentials, security_header) = extract_credentials(url);
    let response = client
        .post(url_without_credentials)
        .body(ptz_move_continuous_start_body(profile_token, direction, &security_header))
        .send()
        .await
        .unwrap();
    let _body = response.text()
        .await
        .unwrap();
    // TODO: parse errors, etc.
}

async fn move_stop(url: &Url, profile_token: &str, client: &reqwest::Client) {
    let (url_without_credentials, security_header) = extract_credentials(url);
    let response = client
        .post(url_without_credentials)
        .body(ptz_stop_body(profile_token, &security_header))
        .send()
        .await
        .unwrap();
    let _body = response.text()
        .await
        .unwrap();
    // TODO: parse errors, etc.
}

fn media_get_profiles_body(security_header: &str) -> String {
    format!(
        r#"
        <s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
            xmlns:media="http://www.onvif.org/ver10/media/wsdl">
            <s:Header>{}</s:Header>
            <s:Body>
                <media:GetProfiles/>
            </s:Body>
        </s:Envelope>
        "#,
        security_header
    )
}

fn create_wsse_token(username: &str, password: &str) -> String {
    let nonce = &rand::random::<[u8; 16]>();
    let nonce_encoded = BASE64_STANDARD.encode(nonce);
    let created = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let digest_bytes = Sha1::new()
        .chain_update(nonce)
        .chain_update(created.as_bytes())
        .chain_update(password.as_bytes())
        .finalize();
    let password_digest = BASE64_STANDARD.encode(digest_bytes.as_slice());
    format!(
        r#"
        <wsse:Security
            xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"
            xmlns:wsu="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd">
            <wsse:UsernameToken>
                <wsse:Username>{}</wsse:Username>
                <wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest">{}</wsse:Password>
                <wsse:Nonce>{}</wsse:Nonce>
                <wsu:Created>{}</wsu:Created>
            </wsse:UsernameToken>
        </wsse:Security>
        "#,
        username, password_digest, nonce_encoded, created
    )
}

fn parse_ptz_capabilities(xml: &str) -> Vec<PtzCapabilities> {
    use xml::reader::{EventReader, XmlEvent};
    use std::io::Cursor;
    let cursor = Cursor::new(xml);
    let parser = EventReader::new(cursor);
    let mut ptz_capabilities = Vec::new();
    let mut current_profile_token = String::new();
    let mut in_ptz_configuration = false;
    let mut supported_movements = Vec::new();
    let mut supported_zoom = Vec::new();

    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                if name.local_name == "Profiles" {
                    for attr in attributes {
                        if attr.name.local_name == "token" {
                            current_profile_token = attr.value;
                        }
                    }
                } else if name.local_name == "PTZConfiguration" {
                    in_ptz_configuration = true;
                } else if in_ptz_configuration {
                    match name.local_name.as_str() {
                        "DefaultAbsolutePantTiltPositionSpace" => {
                            if !supported_movements.contains(&MovementKind::Absolute) {
                                supported_movements.push(MovementKind::Absolute);
                            }
                        }
                        "DefaultRelativePanTiltTranslationSpace" => {
                            if !supported_movements.contains(&MovementKind::Relative) {
                                supported_movements.push(MovementKind::Relative);
                            }
                        }
                        "DefaultContinuousPanTiltVelocitySpace" => {
                            if !supported_movements.contains(&MovementKind::Continuous) {
                                supported_movements.push(MovementKind::Continuous);
                            }
                        }
                        "DefaultAbsoluteZoomPositionSpace" => {
                            if !supported_zoom.contains(&MovementKind::Absolute) {
                                supported_zoom.push(MovementKind::Absolute);
                            }
                        }
                        "DefaultRelativeZoomTranslationSpace" => {
                            if !supported_zoom.contains(&MovementKind::Relative) {
                                supported_zoom.push(MovementKind::Relative);
                            }
                        }
                        "DefaultContinuousZoomVelocitySpace" => {
                            if !supported_zoom.contains(&MovementKind::Continuous) {
                                supported_zoom.push(MovementKind::Continuous);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.local_name == "PTZConfiguration" {
                    ptz_capabilities.push(PtzCapabilities {
                        profile_token: current_profile_token.clone(),
                        supported_movements: supported_movements.clone(),
                        supported_zoom: supported_zoom.clone(),
                    });
                    in_ptz_configuration = false;
                    supported_movements.clear();
                    supported_zoom.clear();
                }
            }
            Err(e) => println!("Error: {}", e),
            _ => {}
        }
    }

    ptz_capabilities
}

fn ptz_move_relative_body(profile_token: &str, direction: PtzDirection, security_header: &str) -> String {
    const DISTANCE: f32 = 0.1;
    let (pan, tilt) = match direction {
        PtzDirection::Up => (0.0, DISTANCE),
        PtzDirection::Down => (0.0, -DISTANCE),
        PtzDirection::Left => (-DISTANCE, 0.0),
        PtzDirection::Right => (DISTANCE, 0.0),
        PtzDirection::ZoomIn => (DISTANCE, 0.0),
        PtzDirection::ZoomOut => (-DISTANCE, 0.0),
    };
    let translation = match direction.is_zoom() {
        true => format!(
            r#"<tt:Zoom x="{:.1}" xmlns:tt="http://www.onvif.org/ver10/schema"/>"#,
            pan
        ),
        false => format!(
            r#"<tt:PanTilt x="{:.1}" y="{:.1}" xmlns:tt="http://www.onvif.org/ver10/schema"/>"#,
            pan, tilt
        )
    };
    format!(
        r#"
        <s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
            xmlns:ptz="http://www.onvif.org/ver20/ptz/wsdl">
            <s:Header>{}</s:Header>
            <s:Body>
                <ptz:RelativeMove>
                    <ptz:ProfileToken>{}</ptz:ProfileToken>
                    <ptz:Translation>
                        {}
                    </ptz:Translation>
                </ptz:RelativeMove>
            </s:Body>
        </s:Envelope>
        "#,
        security_header, profile_token, translation
    )
}

fn ptz_move_continuous_start_body(profile_token: &str, direction: PtzDirection, security_header: &str) -> String {
    const VELOCITY: f64 = 0.1; // some cameras ignore this anyway
    let (x, y) = match direction {
        PtzDirection::Up => (0.0, VELOCITY),
        PtzDirection::Down => (0.0, -VELOCITY),
        PtzDirection::Left => (-VELOCITY, 0.0),
        PtzDirection::Right => (VELOCITY, 0.0),
        PtzDirection::ZoomIn => (VELOCITY, 0.0),
        PtzDirection::ZoomOut => (-VELOCITY, 0.0),
    };
    let velocity = match direction.is_zoom() {
        true => format!(
            r#"<tt:Zoom x="{:.1}" xmlns:tt="http://www.onvif.org/ver10/schema"/>"#,
            x
        ),
        false => format!(
            r#"<tt:PanTilt x="{:.1}" y="{:.1}" xmlns:tt="http://www.onvif.org/ver10/schema"/>"#,
            x, y
        )
    };
    format!(
        r#"
        <s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
            xmlns:ptz="http://www.onvif.org/ver20/ptz/wsdl">
            <s:Header>{}</s:Header>
            <s:Body>
                <ptz:ContinuousMove>
                    <ptz:ProfileToken>{}</ptz:ProfileToken>
                    <ptz:Velocity>
                        {}
                    </ptz:Velocity>
                </ptz:ContinuousMove>
            </s:Body>
        </s:Envelope>
        "#,
        security_header, profile_token, velocity
    )
}

fn ptz_stop_body(profile_token: &str, security_header: &str) -> String {
    format!(
        r#"
        <s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
            xmlns:ptz="http://www.onvif.org/ver20/ptz/wsdl">
            <s:Header>{}</s:Header>
            <s:Body>
                <ptz:Stop>
                    <ptz:ProfileToken>{}</ptz:ProfileToken>
                    <ptz:PanTilt>true</ptz:PanTilt>
                    <ptz:Zoom>true</ptz:Zoom>
                </ptz:Stop>
            </s:Body>
        </s:Envelope>
        "#,
        security_header, profile_token
    )
}
