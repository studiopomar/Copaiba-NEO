<div align="center">

<img src="favicon_mori.png" alt="Ícone do Copaiba NEO" width="104" />

# Copaiba NEO

**Editor multiplataforma de `oto.ini` para UTAU e OpenUtau, escrito em Rust.**

[![Última release](https://img.shields.io/github/v/release/studiopomar/Copaiba-NEO?display_name=tag&style=for-the-badge&logo=github)](https://github.com/studiopomar/Copaiba-NEO/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/studiopomar/Copaiba-NEO/total?style=for-the-badge&logo=github)](https://github.com/studiopomar/Copaiba-NEO/releases)
[![Estrelas](https://img.shields.io/github/stars/studiopomar/Copaiba-NEO?style=for-the-badge&logo=github&color=DAAA3F)](https://github.com/studiopomar/Copaiba-NEO/stargazers)
[![Rust](https://img.shields.io/badge/Rust-2021-orange?style=for-the-badge&logo=rust)](Cargo.toml)
[![Licença](https://img.shields.io/badge/licen%C3%A7a-MIT-blue?style=for-the-badge)](Cargo.toml)

[Baixar](https://github.com/studiopomar/Copaiba-NEO/releases/latest) · [Versão Web](https://studiopomar.github.io/Copaiba-Lexicon-LTS/) · [Recursos](recursos_br.md) · [Histórico de versões](#histórico-de-versões) · [Reportar problema](https://github.com/studiopomar/Copaiba-NEO/issues)

</div>

O Copaiba NEO ajuda a criar e revisar os parâmetros de pronúncia de bancos de voz. A tabela de aliases e a waveform oferecem edição numérica e visual de `Offset`, `Overlap`, `Preutterance`, `Consonant` e `Cutoff`.

> **Versão atual:** **v246 Damasco**. Esta release é compilada para Windows x86_64, macOS universal e Linux x86_64.

## Sumário

- [Recursos](#recursos)
- [Baixar e executar](#baixar-e-executar)
- [Primeiros passos](#primeiros-passos)
- [Auto-Oto com mVocalToolkit](#auto-oto-com-mvocaltoolkit)
- [Versões e lançamentos](#versões-e-lançamentos)
- [Compilar a partir do código](#compilar-a-partir-do-código)
- [Contribuir e reportar problemas](#contribuir-e-reportar-problemas)
- [Star History](#star-history)
- [Créditos e licença](#créditos-e-licença)

## Recursos

- Editar aliases e parâmetros do `oto.ini` na tabela ou diretamente sobre a waveform.
- Abrir bancos de voz e alternar entre vários arquivos em abas.
- Filtrar, selecionar vários aliases e aplicar edições em lote.
- Usar desfazer, refazer, salvamento automático e configurações de tema e idioma.
- Visualizar waveform, espectrograma e curva de pitch; reproduzir amostras e testar um resampler configurado.
- Integrar com o fluxo de trabalho do OpenUtau e importar sequências de fonemas pelo plugin.
- Usar interfaces em português, inglês e árabe, com disposição RTL quando o árabe está selecionado.
- Gerar propostas de alinhamento do `oto.ini` pelo mVocalToolkit e revisar cada alteração antes de aplicar.

## Baixar e executar

Baixe uma versão na página de [Releases](https://github.com/studiopomar/Copaiba-NEO/releases). Os workflows do projeto preparam builds para Windows, Linux e macOS. O app macOS é universal, mas não está assinado nem notarizado; na primeira abertura, use o menu de contexto do Finder e escolha **Abrir**. Consulte as notas e os arquivos anexados à release escolhida para confirmar o que está disponível nela.

Também existe uma [versão Web](https://studiopomar.github.io/Copaiba-Lexicon-LTS/). O Auto-Oto que inicia um serviço local é exclusivo do aplicativo desktop.

## Primeiros passos

1. Abra uma pasta de banco de voz ou selecione um `oto.ini`.
2. Escolha um alias na tabela para carregar sua waveform.
3. Edite os parâmetros na tabela ou arraste os marcadores da waveform.
4. Salve o `oto.ini` e teste a pronúncia no resampler configurado.

## Auto-Oto com mVocalToolkit

O Auto-Oto usa o [mVocalToolkit](https://github.com/Megageorgio/mVocalToolkit) para alinhar fonemas e propor valores para o `oto.ini`. As propostas aparecem para revisão antes de serem aplicadas.

Na primeira execução, o Copaiba pode preparar o runtime e baixar os modelos necessários. É preciso ter acesso à internet para essa etapa. A ferramenta usa um serviço local em `127.0.0.1:8765`; o suporte está disponível no aplicativo desktop.

Para aliases japoneses, informe kana ou romaji. Para outros idiomas, use fonemas separados por espaço no alias. Confira as propostas na waveform, pois o alinhamento não determina sozinho todos os parâmetros específicos de cada banco.

## Versões e lançamentos

| Versão | Codinome | Data | Situação | Notas |
| --- | --- | --- | --- | --- |
| [`v246`](https://github.com/studiopomar/Copaiba-NEO/releases/latest) | Damasco | 21/10/2026 | Release atual | [Notas](changelog.md#v246-damasco---2026-10-21) |
| [`v201`](https://github.com/studiopomar/Copaiba-NEO/releases/tag/v201) | Carambola | 10/05/2026 | Release | [Notas](https://github.com/studiopomar/Copaiba-NEO/releases/tag/v201) |
| [`v190`](https://github.com/studiopomar/Copaiba-NEO/releases/tag/v190) | Bergamota | 11/04/2026 | Release | [Notas](https://github.com/studiopomar/Copaiba-NEO/releases/tag/v190) |
| [`v150_Canary`](https://github.com/studiopomar/Copaiba-NEO/releases/tag/v150_Canary) | Abacaxi | 26/03/2026 | Canary | [Notas](https://github.com/studiopomar/Copaiba-NEO/releases/tag/v150_Canary) |
| [`v140_Canary`](https://github.com/studiopomar/Copaiba-NEO/releases/tag/v140_Canary) | — | 25/03/2026 | Canary | [Notas](https://github.com/studiopomar/Copaiba-NEO/releases/tag/v140_Canary) |
| [`v120_Canary`](https://github.com/studiopomar/Copaiba-NEO/releases/tag/v120_Canary) | — | 24/03/2026 | Canary | [Notas](https://github.com/studiopomar/Copaiba-NEO/releases/tag/v120_Canary) |

Veja o [histórico completo de alterações](changelog.md) e os [lançamentos no GitHub](https://github.com/studiopomar/Copaiba-NEO/releases).

## Compilar a partir do código

É necessário ter uma instalação recente do Rust. Para executar a versão desktop:

```sh
cargo run --release
```

Para compilar sem executar:

```sh
cargo build --release
```

## Contribuir e reportar problemas

Abra uma [issue](https://github.com/studiopomar/Copaiba-NEO/issues) para relatar um defeito ou sugerir uma melhoria. Ao reportar um problema, inclua o sistema operacional, a versão do Copaiba, os passos para reproduzi-lo e, quando possível, um exemplo mínimo do `oto.ini` afetado.

## Star History

[![Histórico de estrelas do Copaiba NEO](https://api.star-history.com/svg?repos=studiopomar/Copaiba-NEO&type=Date)](https://star-history.com/#studiopomar/Copaiba-NEO&Date)

## Créditos e licença

- **Desenvolvimento:** xiao e HAI-D.
- **Garantia de qualidade:** Zone e Makki.
- **Ícone:** Mori-P.
- **Alinhamento Auto-Oto:** [mVocalToolkit](https://github.com/Megageorgio/mVocalToolkit).

O pacote declara a licença MIT em [`Cargo.toml`](Cargo.toml). Consulte também os termos de cada ferramenta externa e dos modelos baixados.
