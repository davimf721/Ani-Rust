# AniRust
![image](https://github.com/user-attachments/assets/cf959020-8933-4fc6-b161-3731494c08e8)

AniRust é um programa de linha de comando em Rust para buscar e reproduzir animes.

## Funcionalidades

- Busca de animes
- Busca automática de fontes de streaming do GoGoAnime
- Histórico de visualização para lembrar onde você parou
- Reprodução de vídeos usando o MPV

## Requisitos

- Rust e Cargo (para compilar o programa)
- MPV (para reprodução de vídeos)
- yt-dlp (para extrair URLs de streaming)

## Instalação

### 1. Instale as dependências

```bash
# Instale o MPV
sudo apt install mpv

# Instale o yt-dlp
pip install yt-dlp
```

### 2. Compile o programa

```bash
# Clone o repositório
git clone https://github.com/seu-usuario/anirust.git
cd anirust

# Compile o programa
cargo build --release

# Opcional: Instale o programa no sistema
cargo install --path .
```

## Uso

### Buscar e assistir um anime

```bash
# Digite o nome do programa
anirust
# Depois escreva o nome do anime
Digite o nome do anime: Naruto
```

O programa irá:
1. Buscar animes correspondentes.
2. Mostrar uma lista de resultados para você escolher
3. Mostrar uma lista de episódios disponíveis
4. Buscar automaticamente o episódio no GoGo
5. Extrair a URL de streaming e reproduzir o vídeo com o MPV
6. Salvar seu progresso no histórico de visualização

## Solução de Problemas

### O programa não encontra o MPV

Certifique-se de que o MPV está instalado e disponível no seu PATH:

```bash
sudo apt install mpv
```

### O programa não encontra o yt-dlp

Certifique-se de que o yt-dlp está instalado e disponível no seu PATH:

```bash
pip install yt-dlp
```

## Contribuição

Contribuições são bem-vindas! Sinta-se à vontade para abrir issues ou enviar pull requests.

## Licença

Este projeto está licenciado sob a licença MIT.
