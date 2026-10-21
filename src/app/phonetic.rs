use std::collections::HashMap;

pub fn get_hiragana_to_romaji() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    // Vowels
    map.insert("あ", "a");  map.insert("い", "i");  map.insert("う", "u");  map.insert("え", "e");  map.insert("お", "o");
    // K-series
    map.insert("か", "ka"); map.insert("き", "ki"); map.insert("く", "ku"); map.insert("け", "ke"); map.insert("こ", "ko");
    // S-series
    map.insert("さ", "sa"); map.insert("し", "shi");map.insert("す", "su"); map.insert("せ", "se"); map.insert("そ", "so");
    // T-series
    map.insert("た", "ta"); map.insert("ち", "chi");map.insert("つ", "tsu");map.insert("て", "te"); map.insert("と", "to");
    // N-series
    map.insert("な", "na"); map.insert("に", "ni"); map.insert("ぬ", "nu"); map.insert("ね", "ne"); map.insert("の", "no");
    // H-series
    map.insert("は", "ha"); map.insert("ひ", "hi"); map.insert("ふ", "fu"); map.insert("へ", "he"); map.insert("ほ", "ho");
    // M-series
    map.insert("ま", "ma"); map.insert("み", "mi"); map.insert("む", "mu"); map.insert("め", "me"); map.insert("も", "mo");
    // Y-series
    map.insert("や", "ya"); map.insert("ゆ", "yu"); map.insert("よ", "yo");
    // R-series
    map.insert("ら", "ra"); map.insert("り", "ri"); map.insert("る", "ru"); map.insert("れ", "re"); map.insert("ろ", "ro");
    // W-series & N
    map.insert("わ", "wa"); map.insert("を", "wo"); map.insert("ん", "n");
    
    // Dakuten (G, Z, D, B)
    map.insert("が", "ga"); map.insert("ぎ", "gi"); map.insert("ぐ", "gu"); map.insert("げ", "ge"); map.insert("ご", "go");
    map.insert("ざ", "za"); map.insert("じ", "ji"); map.insert("ず", "zu"); map.insert("ぜ", "ze"); map.insert("ぞ", "zo");
    map.insert("だ", "da"); map.insert("ぢ", "ji"); map.insert("づ", "zu"); map.insert("で", "de"); map.insert("ど", "do");
    map.insert("ば", "ba"); map.insert("び", "bi"); map.insert("ぶ", "bu"); map.insert("べ", "be"); map.insert("ぼ", "bo");
    // Handakuten (P)
    map.insert("ぱ", "pa"); map.insert("ぴ", "pi"); map.insert("ぷ", "pu"); map.insert("ぺ", "pe"); map.insert("ぽ", "po");
    
    // Combinations (Y-series)
    map.insert("きゃ", "kya"); map.insert("きゅ", "kyu"); map.insert("きょ", "kyo");
    map.insert("しゃ", "sha"); map.insert("しゅ", "shu"); map.insert("しょ", "sho");
    map.insert("ちゃ", "cha"); map.insert("ちゅ", "chu"); map.insert("ちょ", "cho");
    map.insert("にゃ", "nya"); map.insert("にゅ", "nyu"); map.insert("にょ", "nyo");
    map.insert("ひゃ", "hya"); map.insert("ひゅ", "hyu"); map.insert("ひょ", "hyo");
    map.insert("みゃ", "mya"); map.insert("みゅ", "myu"); map.insert("みょ", "myo");
    map.insert("りゃ", "rya"); map.insert("りゅ", "ryu"); map.insert("りょ", "ryo");
    
    // Combinations (Dakuten)
    map.insert("ぎゃ", "gya"); map.insert("ぎゅ", "gyu"); map.insert("ぎょ", "gyo");
    map.insert("じゃ", "ja");  map.insert("じゅ", "ju");  map.insert("じょ", "jo");
    map.insert("びゃ", "bya"); map.insert("びゅ", "byu"); map.insert("びょ", "byo");
    map.insert("ぴゃ", "pya"); map.insert("ぴゅ", "pyu"); map.insert("ぴょ", "pyo");
    
    // Foreign-sound combinations used by extended Japanese VCV banks.
    map.insert("ふぁ", "fa"); map.insert("ふぃ", "fi"); map.insert("ふぇ", "fe"); map.insert("ふぉ", "fo");
    map.insert("ちぇ", "che"); map.insert("しぇ", "she"); map.insert("じぇ", "je");
    map.insert("つぁ", "tsa"); map.insert("つぃ", "tsi"); map.insert("つぇ", "tse"); map.insert("つぉ", "tso");
    map.insert("ゔぁ", "va"); map.insert("ゔぃ", "vi"); map.insert("ゔ", "vu"); map.insert("ゔぇ", "ve"); map.insert("ゔぉ", "vo");
    map.insert("いぇ", "ye");
    
    // Small variations
    map.insert("てぃ", "ti"); map.insert("とぅ", "tu"); map.insert("でぃ", "di"); map.insert("どぅ", "du");
    
    map
}

pub fn get_romaji_to_hiragana() -> HashMap<&'static str, &'static str> {
    let h2r = get_hiragana_to_romaji();
    let mut map = HashMap::new();
    for (h, r) in h2r {
        map.insert(r, h);
    }
    // Add alternatives for Romaji
    map.insert("si", "し");
    map.insert("tu", "つ");
    map.insert("hu", "ふ");
    map.insert("zi", "じ");
    map.insert("vu", "ゔ");
    map
}

pub fn convert_alias(alias: &str, to_hiragana: bool) -> String {
    let map = if to_hiragana { get_romaji_to_hiragana() } else { get_hiragana_to_romaji() };
    
    let input: Vec<char> = alias.chars().collect();
    let mut result = String::new();
    let mut i = 0;
    
    while i < input.len() {
        let mut found = false;
        
        // Try longest matches first (up to 3 chars)
        for len in (1..=3).rev() {
            if i + len <= input.len() {
                let sub: String = input[i..i+len].iter().collect();
                if let Some(&conv) = map.get(sub.as_str()) {
                    result.push_str(conv);
                    i += len;
                    found = true;
                    break;
                }
            }
        }
        
        if !found {
            result.push(input[i]);
            i += 1;
        }
    }
    
    result
}
