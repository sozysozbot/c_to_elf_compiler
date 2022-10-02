fn main() -> std::io::Result<()> {
    let input = std::env::args().nth(1).expect("入力が与えられていません");
    let input: u8 = input.parse().expect("入力をパースできません");
    let tiny_3 = include_bytes!("../experiment/tiny3");
    let tiny_42 = include_bytes!("../experiment/tiny42");
    assert_eq!(tiny_3.len(), tiny_42.len());

    let file = std::fs::File::create("a.out")?;

    {
        use std::io::Write;
        let mut writer = std::io::BufWriter::new(file);
        for (index, byte) in tiny_3.iter().enumerate() {
            if *byte == tiny_42[index] {
                writer.write_all(&[*byte])?;
            } else if *byte == 3 && tiny_42[index] == 42 {
                writer.write_all(&[input])?;
            } else {
                panic!("`../experiment/tiny3` と `../experiment/tiny42` の間に非自明な差分が見つかったので、なにを出力すべきか分かりません")
            }
        }
    }
    Ok(())
}
