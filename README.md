# Soundpad / Bard Minstrel

Aplicacao desktop para streamers que combina um **Soundpad** de efeitos sonoros com um **Bard Minstrel** de musica de fundo, ambos integrados ao OBS Studio via WebSocket.

## Funcionalidades

### Soundpad (SFX)
- Pasta de origem para arquivos `.wav`
- Atalhos de teclado configuraveis por clip
- Volume configuravel por clip
- Toca o audio diretamente no OBS (Media Source)
- Um unico registro de hotkeys entre Soundpad e Bard (sem conflitos)
- Persistencia da configuracao em JSON com escrita atomica

### Bard Minstrel (BGM)
- Pasta de origem para arquivos `.wav`
- Reproducao aleatoria em intervalo de tempo configuravel
- Controle de Play / Pause / Skip
- Ativacao e desativacao do Bard
- Ducking automatico: quando um SFX toca, o volume do BGM abaixa proporcionalmente via filtro Compressor com sidechain no OBS

### Interface (egui)
- Aba **Soundpad**: adicionar/remover clips, definir atalhos e volume
- Aba **Bard Minstrel**: play/pause/skip, volume, intervalo, configuracao de ducking
- Aba **Configuracoes**: host/porta/senha do OBS, nomes das sources, filtro de ducking

## Arquitetura

Projeto estruturado em **Clean Architecture** com separacao clara entre dominio, aplicacao, infraestrutura e UI:

```
src/
├── domain/           # Entidades e regras de negocio puras (sem I/O)
├── application/      # Ports (traits) e Use Cases
├── infrastructure/   # Implementacoes: OBS WebSocket, hotkeys globais, persistencia JSON
└── ui/               # Interface grafica com eframe/egui
```

O dominio e a aplicacao nao dependem de nenhuma crate de infraestrutura — comunicam-se apenas por traits (portas), permitindo trocar `obws` por um plugin nativo do OBS no futuro sem alterar regras de negocio.

## Stack

| Componente | Crate | Versao |
|---|---|---|
| Linguagem | Rust | edition 2021 |
| OBS WebSocket | `obws` | 0.15 |
| Hotkeys globais | `global-hotkey` | 0.8 |
| UI | `eframe` + `egui` | 0.34 |
| Persistencia | `serde` + `serde_json` | 1.x |
| Async runtime | `tokio` | 1.x |
| File dialogs | `rfd` | 0.15 |

## Pre-requisitos

- **Rust** 1.77+ (recomendado via [rustup](https://rustup.rs/))
- **OBS Studio** 28+ com o plugin **obs-websocket** ativo (padrao desde o OBS 28)
- **Linux**: `sudo apt install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

## Como executar

```bash
# Clone o repositorio
git clone https://github.com/silvaalex/soundpad-bard.git
cd soundpad-bard

# Compile e rode
cargo run
```

A janela da aplicacao abrira com tres abas: Soundpad, Bard Minstrel e Configuracoes.

### Configuracao inicial

1. Abra o OBS Studio e va em **Ferramentas > Configuracoes do obs-websocket**
2. Anote a **porta** (padrao: 4455) e a **senha** (se houver)
3. Na aba **Configuracoes** da aplicacao, preencha host, porta e senha
4. Crie uma **Media Source** no OBS para o SFX (ex: "SoundpadSFX") e outra para o BGM (ex: "BardBGM")
5. Coloque os nomes das sources nos campos correspondentes
6. Na aba **Soundpad**, selecione a pasta com seus `.wav`, adicione clips com atalhos e volume
7. Na aba **Bard Minstrel**, selecione a pasta de musicas e configure intervalo/volume

## Como testar

```bash
# Roda os testes unitarios do dominio
cargo test

# Verifica compilacao sem gerar binario
cargo check
```

## Contribuir

1. Fork o repositorio
2. Crie uma branch para sua feature (`git checkout -b feature/nova-feature`)
3. Commit suas alteracoes (`git commit -m 'Adiciona nova feature'`)
4. Push para a branch (`git push origin feature/nova-feature`)
5. Abra um Pull Request

### Convencao de commits

Use mensagens descritivas em portugues ou ingles:
- `adiciona persistencia JSON`
- `corrige conflito de hotkeys`
- `implementa ducking via filtro sidechain`

### Guia de contribuicao por camada

- **`domain/`**: Entidades, value objects e servicos puros. Sem I/O, sem dependencias externas. Fcil de testar isolado.
- **`application/`**: Ports (traits) e use cases. Define o contrato sem saber a implementacao.
- **`infrastructure/`**: Implementacoes concretas dos ports (obws, global-hotkey, JSON).
- **`ui/`**: Interfaces graficas com egui. Cada view e um arquivo separado em `ui/views/`.

## Roadmap

- [ ] Integrar tokio runtime + hotkey listener real no `main.rs`
- [ ] Conectar use cases com a UI (execucao ao vivo)
- [ ] Bard scheduler (tokio task com loop aleatorio)
- [ ] System tray com `tray-icon`
- [ ] Reconexao automatica ao OBS com backoff
- [ ] Normalizacao de volume (LUFS/peak)
- [ ] Perfis de configuracao (modo jogo / modo chat)
- [ ] Suporte a mp3/ogg alem de wav

## Licenca

MIT
