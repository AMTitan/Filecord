use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use flate2::write::ZlibDecoder;
use rust_embed::RustEmbed;
use urlencoding::decode;

#[derive(RustEmbed)]
#[folder = "www"]
struct Asset;

fn main() {
    let listener = TcpListener::bind(format!(
        "0.0.0.0:{}",
        env::var("PORT").unwrap_or("80".to_string())
    ))
        .unwrap();
    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut stream = stream.unwrap();
            let mut total = Vec::new();
            let mut buffer = [0; 4096];
            while stream.read(&mut buffer).unwrap() == 4096 {
                for i in buffer {
                    total.push(i);
                }
            }
            for i in buffer {
                total.push(i);
            }
            let response: String = String::from_utf8_lossy(&total).to_string();
            let url = response
                .split(" ")
                .nth(1)
                .unwrap()
                .chars()
                .collect::<Vec<char>>()[1..]
                .into_iter()
                .collect::<String>()
                .trim()
                .to_string();
            if url == "" || Asset::get(&url).is_some() {
                let url = if url == "" { "index.html" } else { &url };
                let data = Asset::get(url).unwrap().data;
                let mut buf = Vec::new();
                let guess = mime_guess::from_path(url).first_or_octet_stream();
                for i in format!(
                    "HTTP/1.1 200 Ok\r\nContent-Length: {}\r\nContent-Type: {}/{}\r\n\r\n",
                    data.len(),
                    guess.type_(),
                    guess.subtype()
                )
                    .bytes()
                {
                    buf.push(i);
                }
                for i in data.iter() {
                    buf.push(*i);
                }
                stream.write(&buf).unwrap();
            } else {
                let messaging_platforms_useragent =
                    ["bot", "whatsapp", "snapchat", "messenger", "teams", "skype"];
                let mut messaging_platform = false;
                for i in response
                    .split("\r\n\r\n")
                    .nth(0)
                    .unwrap()
                    .to_lowercase()
                    .lines()
                {
                    if i.starts_with("user-agent") {
                        for x in messaging_platforms_useragent {
                            if i.contains(x) {
                                messaging_platform = true;
                                break;
                            }
                        }
                    }
                }
                let char_list: Vec<char> =
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                        .chars()
                        .collect();
                let mut index = 0;
                let mut returns = [0 as u64; 2];
                for x in 0..2 {
                    let mut num: u64 = 0;
                    for y in 0..11 {
                        num += char_list
                            .iter()
                            .position(|c| *c == url.chars().nth(index).unwrap())
                            .unwrap() as u64
                            * (62_u64.pow(y));
                        index += 1;
                    }
                    returns[x as usize] = num;
                }
                let mut buffer = Vec::new();
                ureq::get(&format!(
                    "https://cdn.discordapp.com/attachments/{}/{}/data",
                    returns[0], returns[1]
                ))
                    .call()
                    .unwrap()
                    .into_reader()
                    .read_to_end(&mut buffer)
                    .unwrap();
                let mut writer = Vec::new();
                let mut z = ZlibDecoder::new(writer);
                z.write_all(&buffer).unwrap();
                writer = z.finish().unwrap();
                let text = String::from_utf8(writer.clone()).expect("String parsing error");
                let mut decompressed: Vec<&str> = text.split("&").collect();
                let file_name = decode(decompressed[0]).unwrap();
                decompressed.remove(0);
                let length = decompressed[0];
                decompressed.remove(0);
                if messaging_platform {
                    let send = format!("<!DOCTYPE html>
    <head>
        <title>Sharex - {}</title>
        <meta property=\"og:type\" content=\"website\" />
        <meta name=\"description\" content=\"Sharex is a program to share large files for free using discord\" />
        <meta content=\"#1E293B\" name=\"theme-color\" />
        <link rel=\"apple-touch-icon\" sizes=\"180x180\" href=\"/image/favicon/apple-touch-icon.png\">
        <link rel=\"icon\" type=\"image/png\" sizes=\"32x32\" href=\"/image/favicon/favicon-32x32.png\">
        <link rel=\"icon\" type=\"image/png\" sizes=\"16x16\" href=\"/image/favicon/favicon-16x16.png\">
        <link rel=\"manifest\" href=\"/image/favicon/site.webmanifest\">
        <meta property=\"og:image\" content=\"/image/favicon/sharex.png\" />
    </head>
    <body>", file_name);
                    stream.write(format!("HTTP/1.1 200 Ok\r\nContent-Length: {}\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}", send.len(), send).as_bytes()).unwrap();
                } else {
                    stream.write(format!("HTTP/1.1 200 Ok\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", file_name, length).as_bytes()).unwrap();
                    let mut index = 0;
                    for id in decompressed {
                        let mut buffer = Vec::new();
                        ureq::get(&format!(
                            "https://cdn.discordapp.com/attachments/{}/{}/part_{}",
                            returns[0], id, index
                        ))
                            .call()
                            .unwrap()
                            .into_reader()
                            .read_to_end(&mut buffer)
                            .unwrap();
                        stream.write(&buffer).unwrap();
                        index += 1;
                    }
                }
            }
            stream.flush().unwrap();
        });
    }
}
