# Changelog

## [v190 Bergamota] - 2026-04-11

### English

#### Added

- **Real-time Time-Stretching (WSOLA)**: Implemented high-quality, pure Rust WSOLA (Waveform Similarity Overlap-Add) for smooth playback at different speeds without affecting pitch.
- **Playback Speed Slider**: Added a new UI slider to the header, allowing real-time speed adjustment from 0.25x to 2.0x.
- **Version Tagging**: Standardized internal versioning to "v190 Bergamota" across the UI, Splash Screen, and binaries.

#### Fixed

- **Linux Stability (AppImage)**: Resolved X11 `MaximumRequestLengthExceeded` crashes by optimizing icon sizes and fixed `NoGlutinConfigs` errors by disabling multisampling in the native configuration.
- **CI/CD Optimization**: Improved GitHub Actions workflow with automatic release creation and platform-specific binary naming (including the version tag).

---

### Português

#### Adicionados

- **Time-Stretching em Tempo Real (WSOLA)**: Implementação de algoritmo WSOLA de alta fidelidade em Rust puro, permitindo alterar a velocidade de reprodução sem distorcer o tom (pitch).
- **Slider de Velocidade**: Novo controle deslizante no cabeçalho para ajuste dinâmico da taxa de reprodução entre 0.25x e 2.0x.
- **Padronização de Versão**: Atualização completa para "v190 Bergamota" em todos os elementos da interface, Splash Screen e metadados.

#### Corrigidos

- **Estabilidade no Linux (AppImage)**: Corrigido crash `MaximumRequestLengthExceeded` (redimensionamento de ícone) e erro `NoGlutinConfigs` através da desativação de multisampling.
- **Otimização de CI/CD**: Workflow do GitHub Actions aprimorado com criação automática de releases e nomenclatura inteligente de binários (incluindo a tag de versão).

---

## [v180] - 2026-04-10

### English

#### Added

- **Preset multi-selection (Shortcut Batching)**: Presets (`Ctrl + 1..5`) now apply to all selected rows in the table simultaneously.
- **Improved Scroll Navigation**: Alias scrolling is now based on discrete notches (`raw_scroll_delta`) for higher precision, preventing accidental skips.

#### Fixed

- **Cutoff Textbox Interaction**: Fixed a bug where clicking the Cutoff textbox wouldn't activate direct text entry mode.
- **Stale Waveform/Spectrogram Display**: Resolved issue where navigating between aliases in the same audio file failed to update visuals, causing marker misalignment until a manual zoom.
- **Waveform/Spectrogram UV Tiling**: Eliminated "noise" artifacts (repeated audio chunks) at the right edge of the view during zoom-out by clamping texture coordinates.
- **Spectrogram Out-of-bounds Rendering**: Fixed bright horizontal stripes appearing when the view window extended beyond the actual audio duration.
- **Scroll/Zoom Responsiveness**: Restored smooth, fluid feel for Zoom and Pan while keeping Alias navigation crisp and notch-based.

---

### Português

#### Adicionados

- **Aplicação de Presets em Lote (Multi-seleção)**: Atalhos de presets (`Ctrl + 1..5`) agora são aplicados a todas as linhas selecionadas na tabela simultaneamente.
- **Navegação de Scroll Aprimorada**: Scroll de troca de alias agora utiliza "notches" discretos para maior precisão, prevenindo pulos acidentais de múltiplas entradas.

#### Corrigidos

- **Edição da Caixa de Texto (Cutoff)**: Corrigido bug onde clicar no valor do Cutoff não abria a caixa de edição de texto.
- **Gráficos Obsoletos na Navegação**: Resolvido problema onde navegar entre aliases do mesmo arquivo de áudio não atualizava a Waveform/Espectrograma, causando desalinhamento visual.
- **Artefatos de Repetição (Tiling)**: Eliminado o "ruído" (repetição do início do áudio) no canto direito da tela durante zoom-out através do travamento (clamp) de coordenadas UV.
- **Renderização Fora da Duração**: Corrigido o surgimento de listras horizontais persistentes quando a visualização ultrapassava o final real do arquivo de áudio.
- **Fluidez do Scroll**: Restaurada a aceleração suave para Zoom e Pan, mantendo a troca de aliases precisa e baseada nos "cliques" do scroll.

---

## [v140] - 2026-03-24

### English

#### Added

- **UI Header**: New tab configuration bar displaying character name, path, and image (60x60 with rounded corners).
- **Metadata (Modals)**: Readme and License texts reading through dedicated modal windows.
- **Synthesis Duration**: Custom duration control (default 350ms) in the header for the resampling test.
- **Key Sounds**: Globally configurable random typing sound feedback.
- **Asian Fonts & Emojis**: Full support for rendering Japanese text and Emojis integrated into the main system.
- **Localization (i18n)**: Implemented full multilingual support (i18n) by **HAI-D**.
- **App Icons**: Implemented favicon and system icon created by **Mori-P**.
- **Batch Opening**: Single click on Home Screen groups to automatically load all sub-voicebanks in tabs.
- **Splash Screen**: Redesign with progress messages, stylized logo, fun loading texts, and a credits panel.
- **Audio Recorder (Re-record - F9)**: Allows capturing and replacing an alias's original audio directly via the app. Includes waveform preview.
- **Interaction Lock**: Zooming now requires **CTRL + Scroll** and Panning requires **SHIFT + Scroll**.
- **Loading Bar**: Visual indicator at the top of the Waveform.
- **Multi-Selection support**: Select multiple entries using **Ctrl + Click** or **Shift + Click**.
- **Plugin: Consistency Checker**: Validates voicebanks against logical errors in offset/overlap and missing files.
- **Plugin: Alias Sorter**: Advanced entry sorting (grouping, completed first, alphabetical, etc).
- **Waveform Customization**: Editor color, thickness, and splines mode customization.
- **Spectrogram Customization**: FFT size, Hop Size, Colormaps, Gamma, and Frequency control.
- **Synthesis Test**: App-integrated voice resampling via CLI calls.

#### Fixed

- **Multipitch/Multiplex Voicebanks**: Resolved subfolder loading failures where Character images and info would not load if separated in the voicebank root (Recursive search up to 4 levels for `character.txt`).
- **Dynamic Anchoring of Cutoff**: Fixed waveform logic that failed to anchor positive `Cutoffs` to the end of the audio, causing them to drift when dragging the Offset.
- **Text & Translations**: Live translations and texts (i18n) configured and now visibly activating in all native elements.
- **UI Rendering**: Adjusted bar hierarchy (`TopBottomPanel` and `CentralPanel`) to prevent the Status Bar from clipping the bottom half of the Minimap.
- **Recent UI**: Added a smooth *hover frame* effect to individual items.
- **Major Bug: Undo/Redo**: Undo/Redo now clears corrupted drag/focus states, preventing editing history loss during plugins.
- **Spectrogram/Waveform Alignment**: Solved sample plotting offset issues.
- **Spectrogram Quality**: Prevents stuttering through Bicubic Interpolation (Catmull-Rom) and parallel UI thread lazy computation.
- **Modal Buttons**: Fixed the functional logic of the modal window Close (X) button.
- **Sync: Minimap Colors**: Minimap inherits the primary colors from the clip view window.

#### Refactored

- Remodeled windows into separate modules (`src/app/ui_modals.rs` and `ui_header.rs`).
- Unified Sound and Splash UI correlations to modularize asynchronous initial loading.
- Decoupled spectrogram recomputation from UI render parameter loops.
- Removed residual false positive `dead_code` warnings (clean warnings for lean builds).

---

### Português

#### Adicionados

- **UI Header**: Nova barra de configuração das abas exibindo nome, caminho e imagem do personagem (60x60 com cantos arredondados).
- **Metadados (Modais)**: Leitura de textos (Readme e License) através de janelas modais dedicadas.
- **Duração de Síntese**: Controle de duração personalizado (padrão de 350ms) no header para o resampler.
- **Sons de Teclado**: Feedback sonoro de teclas randômicas configurável globalmente.
- **Fontes Asiáticas e Emojis**: Suporte total à renderização de textos Japoneses e Emojis integrado ao sistema principal.
- **Localização (i18n)**: Implementação do sistema de traduções (i18n) por **HAI-D**.
- **Gráficos e Ícones**: Implementação do favicon e ícone do aplicativo criados por **Mori-P**.
- **Batch Opening**: Clique único em grupos da Home Screen para carregar automaticamente todos os sub-voicebanks abertos em abas.
- **Tela de Inicialização (Splash)**: Redesign com mensagens de progresso, logotipo estilizado, loading text divertido e painel de créditos.
- **Gravador de Áudio (Regravar - F9)**: Permite capturar e substituir o áudio original de um alias diretamente pelo app. Inclui preview de waveform e audição prévia.
- **Interações Travadas (Lock)**: Zoom exige **CTRL + Scroll** e Panning (navegar) exige **SHIFT + Scroll**.
- **Barra de Carregamento**: Indicador visual no topo da Waveform.
- **Multi-Selection support**: Seleção múltipla com **Ctrl + Click** ou **Shift + Click**.
- **Plugin: Consistency Checker (Inspetor)**: Verifica o voicebank para erros lógicos em offset/overlap e arquivos faltantes.
- **Plugin: Alias Sorter**: Ordenação avançada de entradas (agrupamento, concluídos primeiro, alfabética, etc).
- **Waveform Customization**: Customização de cores, espessura e modo de splines do editor sonoro.
- **Spectrogram Customization**: Controle de FFT, Hop Size, Colormaps, Gamma e Frequência.
- **Synthesis Test**: Integração de renderização no aplicativo via chamada CLI.

#### Corrigidos

- **Voicebanks Multipitch/Multiplex**: Resolvido falhas de carregamento de pastas-filhas onde imagens de character e info paravam de carregar se separadas na raiz principal do voicebank. (Busca recursiva de até 4 níveis pelo `character.txt`).
- **Ancoragem Dinâmica de Cutoff**: Consertado lógica da waveform que não amarrava `Cutoffs` positivos ao final do áudio, causando desvios durante o arrasto do Offset.
- **Texto e Traduções**: Traduções e textos em tempo-real (i18n) configurados e agora ativando de forma visível em todos os elementos nativos.
- **Renderização da UI**: Ajustada a hierarquia das barras (`TopBottomPanel` e `CentralPanel`) para prevenir que a Status Bar cortasse metade do Minimapa na parte de baixo.
- **Recentes UI**: Efeito *hover frame* agora reage suavemente aos itens avulsos.
- **Major Bug: Undo/Redo**: Desfazer ou refazer limpa drags/foco corrompidos, prevenindo perda de histórico durante plugins.
- **Spectrogram/Waveform Alignment**: Solucionado desvios de *samples* na plotagem.
- **Qualidade do Espectrograma**: Evita engasgos através de interpolação Bicubica (Catmull-Rom) e lazy computation da UI thread em paralelo.
- **Botões Modais**: Fixado UI de fechamento (X) do sistema de modais.
- **Sync: Cores do Minimapa**: Minimapa herda cores primárias da janela de visualização do clip.

#### Refatorados

- Remodelagem das janelas em módulos separados (`src/app/ui_modals.rs` e `ui_header.rs`).
- Correlações de UI de sons e Splash unificadas para modularizar carregamentos assíncronos iniciais.
- Desacoplamento da recomputacão do espetrograma frente a loops UI render parameters.
- Removidos falsos positivos de `dead_code` residuais (warnings limpos para compilações enxutas).
