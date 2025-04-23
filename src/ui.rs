use anyhow::{Result, Context};
use dialoguer::{Select, Input};

// Função para selecionar um item de uma lista
pub fn select_from_list(items: &[String], prompt: &str) -> Result<usize> {
    let selection = Select::new()
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()
        .context("Falha ao selecionar da lista")?;
    
    Ok(selection)
}

// Função para solicitar entrada de texto
pub fn prompt_input(prompt: &str) -> Result<String> {
    let input = Input::<String>::new()
        .with_prompt(prompt)
        .interact()
        .context("Falha ao obter entrada")?;
    
    Ok(input)
}

// Função para exibir progresso
pub fn show_progress(message: &str) {
    println!("{}", message);
}
