use flate2::write::ZlibDecoder;
use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use urlencoding::decode;

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
            let char_list: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
            let first_length = char_list.iter().position(|c| *c == url.chars().nth(0).unwrap()).unwrap();
            let mut index = 1;
            let mut returns = [0 as u64; 2];
            let mut real_first_length = 0;
            for x in 0..2 {
                let mut num:u64 = 0;
                while (x == 0 && num.to_string().len() < first_length) || (x == 1 && url.len() > index) {
                    num+=char_list.iter().position(|c| *c == url.chars().nth(index).unwrap()).unwrap() as u64*(62_u64.pow(index as u32-1-(x*real_first_length - x*1) as u32));
                    index+=1;
                }
                returns[x as usize] = num;
                real_first_length = index;
            }
            let mut buffer = Vec::new();
            ureq::get(&format!(
                "https://cdn.discordapp.com/attachments/{}/{}/data",
                returns[0],
                returns[1]
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
            let channel = decompressed[0];
            decompressed.remove(0);
            let mut length:usize = 8388608*(decompressed.len()-1);
            let mut buffer = Vec::new();
            ureq::get(&format!(
                "https://cdn.discordapp.com/attachments/{}/{}/part_{}",
                channel, decompressed.last().unwrap(), decompressed.len()-1
            ))
                .call()
                .unwrap()
                .into_reader()
                .read_to_end(&mut buffer)
                .unwrap();
            length += buffer.len();
            drop(buffer);
            stream.write(format!("HTTP/1.1 200 Ok\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n", file_name, length).as_bytes()).unwrap();
            let mut index = 0;
            for id in decompressed {
                let mut buffer = Vec::new();
                ureq::get(&format!(
                    "https://cdn.discordapp.com/attachments/{}/{}/part_{}",
                    channel, id, index
                ))
                    .call()
                    .unwrap()
                    .into_reader()
                    .read_to_end(&mut buffer)
                    .unwrap();
                stream.write(&buffer).unwrap();
                index += 1;
            }
            stream.flush().unwrap();
        });
    }
}