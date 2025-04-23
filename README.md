# AniRust
![image](https://github.com/user-attachments/assets/cf959020-8933-4fc6-b161-3731494c08e8)

AniRust é um programa de linha de comando em Rust para buscar e reproduzir animes.

## Funcionalidades

- Busca de animes usando a API do AniList
- Busca automática de fontes de streaming do AnimeFirePlus
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
# Busque um anime pelo nome
anirustx
```

O programa irá:
1. Buscar animes correspondentes na API do AniList
2. Mostrar uma lista de resultados para você escolher
3. Mostrar uma lista de episódios disponíveis
4. Buscar automaticamente o episódio no AnimeFirePlus
5. Extrair a URL de streaming e reproduzir o vídeo com o MPV
6. Salvar seu progresso no histórico de visualização

### Continuar assistindo de onde parou

```bash
# Continue assistindo o último anime que você estava assistindo
anirust continue
```

O programa irá:
1. Verificar seu histórico de visualização
2. Identificar o último anime que você assistiu
3. Carregar o próximo episódio (ou o mesmo episódio se você já assistiu o último)
4. Buscar automaticamente o episódio no AnimeFirePlus
5. Extrair a URL de streaming e reproduzir o vídeo com o MPV
6. Atualizar seu progresso no histórico de visualização

## Arquivos e Diretórios

- `~/.config/anirust/history.json`: Arquivo de histórico de visualização

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

### O programa não encontra um anime no AnimeFirePlus

O AnimeFirePlus pode ter um nome diferente para o anime. Tente buscar manualmente no site e verificar o nome correto.

## Contribuição

Contribuições são bem-vindas! Sinta-se à vontade para abrir issues ou enviar pull requests.

## Licença

Este projeto está licenciado sob a licença MIT.
