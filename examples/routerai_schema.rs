#![cfg(feature = "derive")]

use anylm::{
    Schema,
    api::{IntoSchema, JsonSchema, Messages},
    completions::{Chunk, Completions},
};
use serde::Deserialize;
use std::collections::HashMap;

/// Описание отдельного навыка
#[derive(Debug, Deserialize, Schema)]
pub struct Skill {
    /// Название навыка (например, "Rust", "Docker")
    pub name: String,

    /// Уровень владения в годах
    #[schema(min = 1, max = 30)]
    pub years_of_experience: u32,
}

/// Информация о контактах
#[derive(Debug, Deserialize, Schema)]
pub struct ContactInfo {
    /// Электронная почта
    pub email: String,

    /// Телефон (необязательное поле)
    pub phone: Option<String>,
}

/// Итоговая структурированная карточка разработчика
#[derive(Debug, Deserialize, Schema)]
pub struct DeveloperProfile {
    /// Полное имя
    pub full_name: String,

    /// Специализация
    #[schema(variants = ["Backend", "Frontend", "Fullstack", "DevOps"])]
    pub role: String,

    /// Контактные данные
    pub contact: ContactInfo,

    /// Список навыков
    pub skills: Vec<Skill>,

    /// Оценка инструментов от 1 до 10
    pub tool_ratings: HashMap<String, u8>,
}

#[atoman::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let messages = Messages::new()
        .user(vec![
            "Создай профиль для Senior Backend разработчика по имени Иван Иванов. ".into(),
            "Он отлично знает Rust (6 лет) и Docker (4 года). ".into(),
            "Контакты: ivan@example.com, телефон не указан. ".into(),
            "Оцени его владение Git на 9 из 10, а Linux на 8 из 10.".into(),
        ])
        .wrap();

    // Генерация структуры JsonSchema через метод трейта IntoSchema::schema()
    let schema: JsonSchema = DeveloperProfile::schema();
    dbg!(&schema);

    let mut response = Completions::openai()
        .base_url("https://routerai.ru/api")
        .read_key("ROUTERAI_API_KEY")?
        .model("qwen/qwen3-coder-next")
        .schema(schema)
        .send(messages)
        .await?;

    let mut full_json_response = String::new();

    while let Some(chunk) = response.next().await {
        if let Chunk::Text(text) = chunk? {
            full_json_response.push_str(&text);
        }
    }
    println!();

    let profile: DeveloperProfile = serde_json::from_str(&full_json_response)?;
    println!("\nРезультат десериализации:");
    println!("{:#?}", profile);

    Ok(())
}
