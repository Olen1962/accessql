# PROJETO: AccesSQL

> Colar este ficheiro no início de cada sessão. No fim de cada sessão, atualizar a secção "Estado atual" e "Registo de sessões".

## 1. O que é

**AccesSQL**: aplicação desktop em Rust (egui/eframe) para gerir bases de dados SQLite locais, com o aspeto do **Microsoft Access 2007-2010** (ribbon, painel de navegação, folha de dados, backstage "Ficheiro").

- Começou como um modelo visual (mock) sem lógica. O aspeto mantém-se; tudo o resto passa a ser real.
- Sem sign-in, sem rede, sem pesquisas online. Tudo local e offline.
- Não usar o nome nem a marca "Microsoft Access" em títulos ou textos da interface. O nome é **AccesSQL**.

## 2. Como trabalhar comigo (instruções para o Claude)

- Responder em **português europeu**.
- O Claude **não acede ao repositório** (o fetch ao GitHub falha). Eu colo os ficheiros relevantes e ele devolve o código.
- O Claude **não compila**. Eu corro `cargo check` / `cargo run` e colo os erros.
- Devolver **ficheiros completos** dos módulos alterados (não só excertos), indicando o caminho (ex.: `src/db.rs`), para eu copiar por cima.
- Quando precisar de um ficheiro que não foi colado, **pedir** em vez de assumir o conteúdo.
- Manter o aspeto Office 2010 existente (`tema.rs`, `widgets.rs`, `icones.rs`). Não redesenhar.
- Um botão sem função real deve ser **removido**, não deixado como "apenas apresentação visual".
- No fim de cada sessão, dar-me a lista dos ficheiros alterados e um resumo para a secção "Registo de sessões".

## 3. Ambiente e stack

- Sistema: FreeBSD (x11). Repositório: https://github.com/Olen1962/accessql (público, ramo `main`, remote por SSH via `gh`).
- Rust edition 2021. `eframe = 0.29` (`default-features = false`, features `default_fonts`, `glow`, `x11`). `rfd = 0.15` para diálogos de ficheiros.
- `rusqlite = { version = "0.32", features = ["bundled"] }` (já no `Cargo.toml`).
- FreeBSD: `sed -i ''` (aspas vazias a seguir ao `-i`).
- Teclado (X11/FreeBSD): o `update` termina com `ctx.output_mut(|o| o.ime = None);`. Sem isso o método de entrada (XIM) engole o Backspace e as setas nas caixas de texto. Solução copiada do projeto `bsdconqueror` (mesmas versões: eframe/egui 0.29.1, winit 0.30.13). Testar acentos (ç, ã) depois de aplicar.
- O egui trazia atalhos de zoom (Ctrl +, Ctrl -, Ctrl 0) que consumiam o Ctrl+' em teclados portugueses (a tecla do apóstrofo é fisicamente a do «menos»). `update` faz `ctx.options_mut(|o| o.zoom_with_keyboard = false)` e o código aceita `Key::Quote` ou `Key::Minus` com Ctrl como «copiar o valor de cima». O zoom global passou a fazer-se com Ctrl + roda do rato: `app.rs` soma os `Event::MouseWheel` com `modifiers.ctrl`, aplica 1,1 por passo e limita o fator entre 0,5 e 2,5.
- Ao aplicar pacotes, confirmar com `git status` que não há mistura de passos; se houver dúvidas, aplicar `accessql_src_completo.zip` (todos os `.rs` do `src/`; não inclui o `Cargo.toml`).
- Depois de cada `unzip` fazer **`touch src/*.rs`** antes do `cargo run`: se os relógios do Claude e do utilizador não coincidirem, o `cargo` julga os ficheiros antigos e não recompila (confirmar que aparece «Compiling accessql»).
- Grelha e ribbon são **desenhados à mão** com `painter` (não usar `egui_extras`).
- `.gitignore` tem de incluir `target/` (o binário passa dos 100 MB e o GitHub rejeita o push).
- Pacote já renomeado para `accessql`.

## 4. Estrutura do código (`src/`)

| Ficheiro | Papel |
|---|---|
| `main.rs` | Janela eframe, declara os módulos |
| `app.rs` | `update()` por frame: QAT, separadores, barra de estado, backstage ou ribbon + navegação + área de trabalho |
| `estado.rs` | `AccessApp` (estado global), `ObjKind` (`Table`/`Query`), `Doc` (com `TableData`), `NavGroup`, `Edicao`, `refresh_nav`, `open_doc` |
| `ribbon.rs` | Barra de título/QAT, separadores e grupos do ribbon |
| `navegacao.rs` | Painel de navegação |
| `area_trabalho.rs` | Separadores de documentos e folha de dados (edição, `(Novo)`, eliminar, atualizar, linhas visíveis) |
| `backstage.rs` | Separador Ficheiro: Informações, Recente, Novo; ações Abrir, Guardar Cópia Como, Fechar, Sair |
| `db.rs` | Camada SQLite (`Database`, `TableData`, `Campo`): open/create, catálogo, load, update/insert/delete, `VACUUM INTO`, compactar, parser do `CREATE TABLE` e preservação de `UNIQUE`/`CHECK`/`AUTOINCREMENT` |
| `eliminar_obj.rs` | Eliminar tabelas e consultas: confirmação, proteção das tabelas referidas por relações, aviso das consultas afetadas |
| `relacoes.rs` | Janela «Relações»: chaves estrangeiras existentes, interruptor da integridade referencial, verificação |
| `totais.rs` | Janela «Totais»: resumo da coluna selecionada |
| `indices.rs` | Janela «Índices»: ver, criar e eliminar índices de uma tabela |
| `desfazer.rs` | Desfazer/Refazer: histórico de alterações a células, registos criados e eliminados |
| `csvio.rs` | Dados Externos: importar CSV (separador e codificação detetados, tabela nova ou acrescentar) e exportar CSV |
| `filtros.rs` | Ordenar, filtrar e procurar na folha de dados: ordenação e filtros por separador, menu da coluna, procura em memória |
| `renomear.rs` | Mudar o nome de tabelas e consultas (F2 no painel de navegação): diálogo e atualização dos separadores abertos |
| `estrutura.rs` | Vista Estrutura: ver campos de uma tabela, acrescentar/renomear/eliminar/alterar campos, desenhar e criar tabelas novas; botão «Vista» que alterna folha de dados/estrutura |
| `consulta.rs` | Vista SQL: editor, executar (F5/Ctrl+Enter), resultados, guardar como `VIEW`, abrir `VIEW` para editar |
| `ficheiro.rs` | Novo, abrir, guardar cópia, fechar, compactar (métodos de `AccessApp`) |
| `recentes.rs` | Lista de recentes em `~/.config/accessql/recentes.txt` |
| `validacao.rs` | Validação dos valores ao gravar (tipos, obrigatório, datas, Sim/Não) e testes unitários |
| `status.rs` | Barra de estado |
| `tema.rs` | Paleta e constantes de cor |
| `widgets.rs` | Botões grandes/pequenos, grupos do ribbon |
| `icones.rs` | Ícones desenhados em código, incluindo o símbolo próprio do AccesSQL (`Icon::Logo`) |


## 5. Decisões tomadas

- Motor de dados: **SQLite** via `rusqlite`. Cada base de dados é um ficheiro SQLite normal.
- Extensão dos ficheiros: **`.accsql`** (SQLite por dentro; não usar `.accdb`, que o Access não abriria).
- Objetos do painel de navegação: **Tabelas** e **Consultas**. As consultas guardadas são **`VIEW`s** do SQLite.
- **Formulários e Relatórios ficam fora** da primeira versão (remover `Form`/`Report` de `ObjKind` e as vistas `form_view`/`report_view`).
- Vistas: Folha de Dados, Estrutura, **SQL** (editor + resultados).
- Eliminar tabelas e consultas: confirmação com o nome, o n.º de registos e aviso de que não se anula; **bloqueia-se só a tabela referida** por chaves estrangeiras de outras (a tabela que guarda a relação pode ser eliminada, e com ela a relação), mesmo com a integridade desligada; avisa das consultas que mencionam o objeto (não bloqueia). Delete no painel de navegação ou «Eliminar» no menu de contexto. Fecha os separadores do objeto e limpa o histórico de desfazer.
- Integridade referencial: `PRAGMA foreign_keys = ON` ao abrir ou criar uma base; a janela «Relações» tem um interruptor (vale só para a sessão). Chaves estrangeiras escolhem-se no campo «Refere a» (só chaves primárias simples) ao desenhar uma tabela nova ou ao acrescentar um campo; usam `REFERENCES tabela(campo)` na definição do campo. Não é possível acrescentar relações a campos que já existem (exigiria recriar a tabela). O erro `FOREIGN KEY constraint failed` é traduzido.
- Cada objeto (tabela ou consulta) tem **um só separador**. O botão «Vista» e o menu de contexto trocam o modo no mesmo separador: tabela = dados <-> estrutura; consulta guardada = dados <-> SQL. Abrir (duplo clique) uma tabela já aberta só a ativa, ou volta ao modo de dados. Se uma consulta tem alterações de SQL por guardar, não se muda de vista (pede-se para guardar).
- Desfazer/Refazer: cada gravação vai para `hist_pend`; `fechar_passo` junta tudo o que se fez numa operação (uma colagem grande desfaz-se de uma vez). Guardam-se os valores brutos do SQLite (inclui BLOB e NULL). O histórico (máx. 200 passos) limpa-se ao fechar/abrir a base de dados, executar SQL que altere dados, mudar a estrutura, mudar o nome de objetos e importar CSV. Não há «Colar» no ribbon: o egui só entrega o texto da área de transferência com Ctrl+V.
- Folha de dados, células: Ctrl+C copia a célula (ou o registo inteiro, em texto separado por tabuladores, se estiver selecionado pelo seletor); Ctrl+X copia e limpa; Ctrl+V cola: na linha `(Novo)` cada linha de texto cria um registo novo (um registo inteiro copiado da grelha duplica-se com ID novo: o campo INTEGER PRIMARY KEY não é escrito); nos registos existentes os valores substituem as células a partir da selecionada (com o registo selecionado, a partir da 1.ª coluna), e linhas a mais que a tabela são ignoradas; Ctrl+' copia o valor da mesma coluna no registo de cima, com a célula selecionada ou durante a edição. Delete limpa a célula (NULL).
- CSV: ao importar deteta-se o separador (`; , tab |`) e a codificação (UTF-8, senão Windows-1252). A tabela nova infere INTEGER/REAL/TEXT por coluna (zeros à esquerda = texto; vírgula decimal opcional). Acrescentar a uma tabela existente liga colunas pelo nome do cabeçalho (ou por posição, sem cabeçalho). A importação é uma só transação: se uma linha falhar, nada é gravado. Exportar grava o que está no separador (com a ordenação e os filtros), UTF-8 com BOM opcional, linhas CRLF.
- Validação ao gravar: antes de `UPDATE`/`INSERT`, colagem ou importação CSV, o valor é verificado contra o tipo declarado e a obrigatoriedade da coluna (`PRAGMA table_info` → `TableData.tipos`/`obrig`). Inteiros, reais (com `.` ou `,`), Sim/Não e datas (`AAAA-MM-DD` ou `DD/MM/AAAA`, com validação de dias e anos bissextos) exigem formato válido; `TEXT`, `BLOB` e tipos personalizados aceitam texto livre. Um erro invalida toda a colagem/importação, que não grava nada. `UNIQUE`, `CHECK` e chaves estrangeiras continuam a ser validados pelo SQLite; as mensagens `UNIQUE`, `CHECK` e `AUTOINCREMENT` são traduzidas para português.
- Ordenar e filtrar: por separador (`Doc.ordem`, `Doc.filtros`, `filtro_ativo`); a ordenação usa `ORDER BY col COLLATE NOCASE`; filtros acumulam com AND (igual, diferente, contém, não contém; vazio = NULL ou ''). Menu da coluna: clique na seta do cabeçalho, botão direito no cabeçalho, ou botão «Filtro» do ribbon. O cabeçalho mostra ▲/▼ na coluna ordenada e fica laranja nas filtradas. «Procurar» (barra de registos) é uma procura em memória nas linhas carregadas, com realce; Enter = seguinte.
- F2: quem o recebe depende do último clique (`foco_nav`): no painel de navegação muda o nome da tabela/consulta selecionada; na folha de dados edita a célula; na Vista Estrutura muda o nome do campo selecionado (ou vai para a caixa do nome, numa tabela nova). Ignorado enquanto se escreve numa caixa de texto conhecida (`a_escrever`).
- Vista Estrutura: tabela nova = grelha editável (nome, tipo, obrigatório, valor por omissão, chave primária, único, expressão CHECK e «Auto») + «Criar Tabela» (uma chave primária INTEGER com «Auto» vira `INTEGER PRIMARY KEY AUTOINCREMENT`; várias chaves viram `PRIMARY KEY (a, b)`). Há ainda um painel «Restrições de tabela» com uma restrição `UNIQUE (...)` ou `CHECK (...)` por linha. Tabela existente = campos com Renomear, **Alterar** (tipo, obrigatório, omissão, único, CHECK e Auto; com aplicação explícita), Eliminar (confirmação) e «Acrescentar campo» (campo obrigatório exige valor por omissão; aceita CHECK, mas recusa chave primária, UNIQUE e AUTOINCREMENT — o SQLite não os permite em `ADD COLUMN`; para UNIQUE existe a janela Índices). As restrições de tabela existentes aparecem só para consulta. Alterar um campo recria a tabela dentro de uma transação (`Database::alterar_campo`): copia todos os dados, recria índices e gatilhos, preserva ações ON DELETE/ON UPDATE, o nome da tabela, as vistas, as restrições inline e as restrições de tabela `UNIQUE`/`CHECK`; se algo falhar, nada muda. O texto original do `CREATE TABLE` é analisado (`segmentos_da_tabela`, `aplicar_restricoes_inline`, `restricoes_tabela`) para recuperar o que `PRAGMA table_info` não devolve. Campos com valor vazio impedem torná-los obrigatórios.
- Vista SQL: um separador por consulta («Consulta1», …). Scripts com várias instruções são executados em sequência (sem transação própria); mostra-se o resultado da última que devolve linhas (máx. 100000). «Guardar como Consulta» cria uma `VIEW` (só `SELECT`); se o separador já veio de uma consulta guardada, substitui-a sem perguntar. Consultas e resultados SQL não têm linha `(Novo)` e não são editáveis.
- Folha de dados: clicar no seletor de registo (coluna esquerda) seleciona o registo; Delete elimina (com confirmação). Com uma célula selecionada, Delete limpa-a, Backspace ou F2/Enter/escrever editam. Ctrl+setas = primeiro/último registo e campo.
- Ao arrancar não há base de dados aberta e o backstage abre em «Recente» (ou «Novo», se não houver recentes).
- Recentes em `~/.config/accessql/recentes.txt` (máx. 10). Abrir recusa ficheiros que não sejam SQLite.
- «Guardar» não existe: o SQLite grava cada alteração logo. «Guardar Cópia Como» usa `VACUUM INTO`.
- Alterações menores são decididas dinamicamente à medida que o projeto avança.

## 6. Âmbito do ribbon (o que fica)

- **Ficheiro**: Novo, Abrir, Guardar Cópia Como, Recentes, Fechar, Sair.
- **Base**: Vistas (Folha de Dados / Estrutura / SQL), Cortar/Copiar (o Colar só por Ctrl+V), Ordenar e Filtrar (`ORDER BY` / `WHERE`), Registos (Novo, Eliminar, Atualizar, Totais), Localizar. Saem: Guardar (a gravação é automática), Pincel de Formatos, Formatação de Texto, Ortografia.
- **Criar**: Tabela, Estrutura da Tabela, Consulta SQL. Saem: Modelos, Partes da Aplicação, Formulários, Relatórios, Macros e Código.
- **Dados Externos**: Importar CSV, Exportar CSV. Saem: Excel, Outlook, dBASE, XML, HTML, Word.
- **Ferramentas de Bases de Dados**: Relações (chaves estrangeiras), Índices, Compactar e Reparar (`VACUUM` + `integrity_check`). Saem: Documentador, Analisador, Mover Dados.
- **Folha de Dados** (separador contextual): Novo campo, Eliminar campo, Renomear. A validação de tipos e `NOT NULL` é automática ao gravar; `UNIQUE`, `CHECK` e relações ficam a cargo do SQLite. O tipo e a alteração de campos existentes tratam-se na Vista Estrutura.

## 7. Identidade (a aplicar)

- Título da janela: **AccesSQL**. `with_app_id("accessql")`.
- Barra de título: «Base1 : Base de Dados SQLite - AccesSQL».
- Ícone próprio: cilindro de base de dados azul desenhado em código (`Icon::Logo` em `icones.rs`), usado na barra de acesso rápido no lugar do «A» vermelho. Não se usa imagem nem marca do Access.
- Nome mantido: **AccesSQL**. Decisão de 2026-11-19: o nome está consolidado no repositório, no binário e em `~/.config/accessql`; mudá-lo agora traria risco e pouca vantagem. A distinção face ao Access faz-se pelo ícone próprio e pela nota de não afiliação no `README.md`. Se o utilizador pedir outro nome, a mudança é pontual.
- `README.md` com descrição, compilação na FreeBSD e nota de não afiliação à Microsoft.
- Licença **GPL-3.0-or-later**, texto oficial em `LICENSE`.

## 8. Problemas conhecidos

Resolvidos nos passos 1 e 2: dados fixos, `OBJECTS` constante, `Doc` sem dados, desenho de todas as linhas, pânico com 0 colunas/linhas, falta de edição.

Em aberto:

1. Resolvido: os valores entram como texto e o SQLite converte pela afinidade, mas agora há validação prévia de tipo/formato (`validacao.rs`).
2. O diálogo de ficheiros (`rfd`) pode não abrir na FreeBSD; se falhar, acrescentar um campo de caminho no backstage.
3. Resolvido: avisos de código não usado e botões sem função removidos; `cargo check` sem avisos.
4. Resolvido: a base de demonstração e `dados_exemplo.rs` foram removidos (dados em memória não eram gravados).
5. Resolvido: as restrições inline `CHECK`/`UNIQUE`, o `AUTOINCREMENT` e as restrições de tabela `UNIQUE`/`CHECK` são lidas do `CREATE TABLE` original e reaplicadas ao alterar um campo. Chaves primárias compostas e relações continuam a ser regeneradas a partir do catálogo (ações ON DELETE/ON UPDATE já preservadas).

## 9. Plano (ordem de trabalho)

1. `db.rs` (SQLite) e painel de navegação a ler o catálogo.
2. Folha de dados a ler e a gravar (edição, linha `(Novo)`, só linhas visíveis).
3. Ficheiro → Novo / Abrir / Guardar (com `rfd`).
4. Vista SQL (editor + resultados) e consultas guardadas como `VIEW`.
5. Vista Estrutura (campos, tipos, validação).
6. Importar/Exportar CSV, Relações, Compactar e Reparar.
7. Limpeza final de ribbon e menus (remover tudo o que não tenha função).

## 10. Estado atual

- [x] Modelo visual completo e a correr (ribbon, navegação, folha de dados, backstage).
- [x] Repositório no GitHub com histórico limpo (sem `target/`).
- [x] Âmbito e decisões fechados (secções 5 e 6).
- [x] Passo 1: `db.rs` + `rusqlite`, navegação ligada ao catálogo, folha de dados a ler (testado, tag `passo1`).
- [x] Passo 2: edição de células, `(Novo)`, Eliminar com confirmação, Atualizar, linhas visíveis (testado, tag `passo2`).
- [x] Passo 3: Ficheiro → Novo / Abrir / Guardar Cópia Como / Fechar / Recentes / Compactar (compilado e testado: criar, abrir, guardar cópia, erros, persistência).
- [x] Passo 3b: seletor de registo, Delete, menu de contexto, teclado da grelha, Compactar no ribbon (testado; tag `passo3` no GitHub).
- [x] Passo 4: Vista SQL (Criar → Consulta SQL / Tabela com modelo; scripts; resultados só de leitura; Guardar como Consulta = `VIEW`; «Vista SQL» no menu de contexto das consultas) e correção do teclado (IME). Testado.
- [x] Passo 5: Vista Estrutura (Criar → Estrutura da Tabela; botão «Vista» no ribbon; menu de contexto «Vista Estrutura» nas tabelas; acrescentar/renomear/eliminar campos com `ALTER TABLE`). Testado.
- [x] Passo 5b (melhorias pedidas, assumido testado): um só separador por objeto (a mudança de vista troca o modo no mesmo separador), F2 universal (célula, campo na estrutura, tabela/consulta no painel de navegação, com «Mudar o Nome» também no menu de contexto): escrito, **por compilar**.
- [x] Passo 5c (testado): ordenar (A-Z/Z-A), filtrar (igual, diferente, contém, não contém), alternar/remover filtros, procurar na barra de registos com realce, menu na seta do cabeçalho e no botão direito: escrito, **por compilar**.
- [x] Passo 6a: Dados Externos (importar e exportar CSV). Testado.
- [x] Passo 6b-1: copiar/colar entre células (Ctrl+C, Ctrl+X, Ctrl+V) e Ctrl+' (valor da célula de cima). Testado.
- [x] Passo 6b-1b: colar vários campos e vários registos. Testado.
- [x] Passo 7a (Editar): Desfazer/Refazer (Ctrl+Z, Ctrl+Y, QAT), botões Cortar/Copiar do ribbon. Testado.
- [x] Passo 7b: limpeza do ribbon Base/Ferramentas/Folha de Dados, janela Totais, janela Índices, Novo/Renomear/Eliminar Campo a partir da folha de dados. Testado.
- [x] Passo 7c: Relações (janela, `PRAGMA foreign_keys = ON`, campo «Refere a» na Vista Estrutura). Testado.
- [x] Passo 7d: eliminar tabelas e consultas (confirmação, bloqueio por relações, aviso de consultas afetadas). Compilado; teste manual da janela pendente.
- [x] Validação ao gravar: `validacao.rs` (tipos, obrigatório, datas, Sim/Não), ligada à edição de células, colagem e importação CSV; 6 testes unitários (`cargo test`).
- [x] Limpeza de controlos sem função (Ajuda, Guardar, cartão «Base de Dados em Branco» agora ligado, cabeçalho de navegação, «Clique para Adicionar») e de código morto; `cargo check` sem avisos.
- [x] Identidade e licença: `README.md`, `LICENSE` (GPL-3.0-or-later), push de `main` e da tag `passo7`.
- [x] Ícone próprio (`Icon::Logo`) no lugar do «A» vermelho; nome mantido como AccesSQL por decisão registada.
- [x] Alterar tipo/obrigatório/omissão de campos existentes: `Database::alterar_campo` (recriação transacional) + botão «Alterar» na Vista Estrutura; 3 testes unitários.
- [x] Remoção da base de demonstração: `Database::demo()`, `abrir_demo()`, cartão do backstage e `dados_exemplo.rs`.

- [x] Passo 8 (commit/push `c297f7b`, tag `passo8`): ícone próprio, botão «Alterar» na Vista Estrutura e build release (`cargo build --release --locked`; `target/release/accessql`, ELF FreeBSD x86-64, 22 MB).
- [x] Ctrl + roda do rato: zoom global da interface (0,5×–2,5×), escrito em `app.rs` e verificado com `cargo check`.
- [x] Passo 9: `Campo.unico/check/autoincrement`, parser do `CREATE TABLE` original, preservação das restrições ao alterar campos, controlos na Vista Estrutura e 4 testes unitários novos (`cargo test`: 13/13).

**Próximo passo:** teste manual confirmado pelo utilizador em 2026-11-19 (zoom Ctrl+roda e preservação das restrições ao alterar campos). Commit/push com tag `passo9` efetuados.

## 11. Registo de sessões

_(Acrescentar uma linha por sessão: data, o que foi feito, ficheiros alterados, o que ficou por fazer.)_

- 2026-11-05: análise do mock, definição de âmbito e decisões, criação deste ficheiro. Sem alterações ao código.
- 2026-11-05 b: passo 1 escrito (sem compilar). Novo `db.rs` (SQLite, base de demonstração em memória com `Clientes`/`Encomendas`/`Produtos` + 2 `VIEW`s); `estado.rs` (`NavGroup`, `Doc.data`, `refresh_nav`, `open_doc` carrega da BD; `ObjKind` só `Table`/`Query`); `navegacao.rs` lê o catálogo; `area_trabalho.rs` (datasheet lê `TableData`, removidos `form_view`/`report_view`, corrigido pânico com 0 colunas/0 linhas); `status.rs`; `ribbon.rs` (título AccesSQL); `main.rs`; `Cargo.toml` (nome `accessql`, `rusqlite`). Por fazer: `cargo check`, depois passo 2.
- 2026-11-05 c: passo 2 escrito (sem compilar). `db.rs` (rowid, update/insert/delete/fetch_row, `ID INTEGER PRIMARY KEY` na demo); `estado.rs` (`Edicao`, `scroll_sel`, `confirm_del`, `Doc.data` sem `Rc`); `area_trabalho.rs` (nova `datasheet` com edição, teclado, linhas visíveis, cabeçalho fixo; `gravar_celula`, `novo_registo`, `pedir_eliminar`, `dialogo_eliminar`, `recarregar_doc`); `ribbon.rs` (Registos ligado); `navegacao.rs` e `area_trabalho.rs` (ids dos campos de texto); `app.rs`. Por testar: edição, Tab/Enter, `(Novo)`, Eliminar.
- 2026-11-05 d: passos 1 e 2 compilados, testados e guardados no git (tags `passo1`, `passo2`). Passo 3 escrito (sem compilar): `db.rs` (`open`/`create`/`copy_to`/`compact`, erros em português), `ficheiro.rs` e `recentes.rs` (novos), `backstage.rs` reescrito (Informações reais, Recente, Novo com pasta e demo), `estado.rs` (`db_file`, `recentes`, `new_db_dir`; arranca sem BD e com o backstage aberto), `ribbon.rs` (título «AccesSQL» sem BD), `main.rs`. Por testar tudo isto.
- 2026-11-05 e: passo 3 testado com sucesso (novo, demo, guardar cópia, reabrir, erros, persistência, ID alterado). Passo 3b escrito (sem compilar): `area_trabalho.rs` (seletor de registo, Delete/Backspace/Ctrl+setas/Home/End/PgUp/PgDn, menu de contexto Novo/Eliminar Registo, Enter confirma o diálogo, id fixo do editor), `estado.rs` (`row_sel`), `ribbon.rs` («Compactar e Reparar» ligado). Dúvida em aberto: Backspace e setas dentro do editor de célula.
- 2026-11-05 f: relatado: Backspace e setas não fazem nada dentro do editor de célula; e uma mensagem «Esta vista não permite eliminar registos» ao tentar eliminar (objeto aberto por confirmar: tabela ou consulta?). Passo 3c = 3b + diagnóstico (`ACCESSQL_DEBUG=1` mostra na barra de estado as teclas recebidas e se o editor tem o foco). À espera do resultado.
- 2026-11-05 g: causa da mensagem «Esta vista não permite eliminar registos» encontrada: `datasheet` faz `mem::take` dos dados do documento e `pedir_eliminar`/`novo_registo` (via Delete e menu de contexto) corriam a meio, com `editable=false`. Corrigido no passo 3d (ações adiadas para depois de devolver os dados). **Regra:** dentro de `datasheet`, não chamar métodos que leiam `docs[active].data` antes da linha que os devolve. Backspace/setas no editor: ainda por diagnosticar (`ACCESSQL_DEBUG=1`).
- 2026-11-05 h: Delete e menu de contexto continuaram a dar «Esta vista não permite eliminar registos» com o ficheiro 3d correto (grep confirmado). Hipótese: build não recompilada (datas dos ficheiros do zip). Pedido `touch src/*.rs` + confirmar «Compiling accessql». Se persistir: acrescentar diagnóstico em `pedir_eliminar`.
- 2026-11-06: passo 3 guardado no git (tag `passo3`; commit de `PROJETO.md` na raiz; `*.accsql` no `.gitignore`). Passo 4 escrito (sem compilar): `db.rs` (`executar`, `dividir`, `view_sql`, `guardar_vista`, `ler` partilhado, `ResultadoSql`, `TableData.truncado`), `consulta.rs` (novo), `estado.rs` (`ObjKind::Sql`, campos `sql`/`info`/`vista` em `Doc`, `pedir_nome`, `n_consulta`), `area_trabalho.rs` (sem `(Novo)` em vistas, `record_bar` com `novo`), `navegacao.rs` (menu «Vista SQL»), `ribbon.rs` (Criar reduzido a Tabela e Consulta SQL), `status.rs`, `app.rs`, `ficheiro.rs`, `main.rs`.
- 2026-11-06 b: passo 4 testado: «Guardar como Consulta» só aceita `SELECT` (por desenho: uma `VIEW` não pode ser um `INSERT`). Backspace e setas continuam sem funcionar nos editores de texto. Passo 4b = `app.rs` com diagnóstico global (`ACCESSQL_DEBUG=1` mostra na barra de estado todas as teclas e texto que chegam ao egui). Passo 4 commitado em `main` (2d63c04); falta `git tag passo4` e `git push`.
- 2026-11-06 c: causa do Backspace/setas encontrada (XIM/IME). Passo 4c = `app.rs` com `ctx.output_mut(|o| o.ime = None)` no fim do `update` (mais o diagnóstico `ACCESSQL_DEBUG`). Por testar: Backspace e setas no editor SQL e nas células; acentos.
- 2026-11-06 d: passo 4 testado e a funcionar, incluindo Backspace e setas (correção do IME). Falta só `git commit` + `git tag passo4` + `git push` do pacote 4c.
- 2026-11-06 e: passo 4 guardado no git (a tag `passo4` apontava ao commit antigo; `git tag -f passo4` + `git push --force origin passo4`). Passo 5 escrito (sem compilar): `db.rs` (`Campo`, `TIPOS`, `info_tabela`, `criar_tabela`, `acrescentar_campo`, `renomear_campo`, `eliminar_campo`), `estrutura.rs` (novo), `estado.rs` (`ObjKind::Design`, `Doc.estr`, `n_tabela`), `consulta.rs`, `status.rs`, `ficheiro.rs`, `area_trabalho.rs`, `navegacao.rs` (menu «Vista Estrutura»), `ribbon.rs` (grupo Vistas com botão que alterna; Criar → «Estrutura da Tabela»), `main.rs`.
- 2026-11-06 f: passo 5 testado (tudo ok). Passo 5b escrito (sem compilar): `estado.rs` (`procurar_objeto`, `open_doc` troca o modo no mesmo separador, `Doc.sql_orig`/`por_guardar`, `a_escrever`, `foco_nav`, `nav_rect`, `renomear_obj`), `consulta.rs` (`abrir_vista_sql` troca no mesmo separador), `estrutura.rs` (seleção de campo, F2, `alternar_vista` também para consultas, rótulo do botão Vista), `renomear.rs` (novo), `db.rs` (`renomear_objeto`), `navegacao.rs` («Mudar o Nome (F2)», `nav_rect`), `area_trabalho.rs`, `app.rs` (foco do último clique, F2 no painel, diálogo), `ficheiro.rs`, `main.rs`.
- 2026-11-06 g: passo 5c escrito (sem compilar): `db.rs` (`Op`, `Condicao`, `load_filtrado`, `ler` com parâmetros), `filtros.rs` (novo), `estado.rs` (`Doc.ordem/filtros/filtro_ativo`, `menu_col`, `focar_busca`), `area_trabalho.rs` (cabeçalho clicável, indicadores, realce de procura, menu de contexto «Filtrar por Seleção»/«Excluir Seleção», barra de registos), `ribbon.rs` (Ordenar e Filtrar e Localizar ligados), `app.rs`, `estrutura.rs`, `consulta.rs`, `main.rs`. Licença: decidir no fim (a favor: GPL-3.0 se não se quiser que o código seja fechado e vendido por terceiros; MIT/Unlicense se for indiferente).
- 2026-11-06 h: passos 5b e 5c testados e a funcionar. Passo 6a escrito (sem compilar): `csvio.rs` (novo: análise de CSV, deteção de separador/codificação, diálogos de importação e exportação), `db.rs` (`inserir_linhas`, `eliminar_tabela`), `ribbon.rs` (Dados Externos reduzido a Importar/Exportar CSV), `estado.rs`, `app.rs`, `ficheiro.rs`, `estrutura.rs` (`recarregar_abertos` público), `main.rs`.
- 2026-11-07: passo 6a testado (um ID repetido não grava nada; importar para tabela vazia e para tabela nova funcionam). Passo 6b-1 escrito (sem compilar): `area_trabalho.rs` (Ctrl+C/X/V, Ctrl+' na grelha e no editor da célula).
- 2026-11-07 b: ao testar o 6b os ficheiros do `src/` estavam misturados de passos diferentes (erros de compilação em `Condicao`, `foco_nav`, etc.). Entregue `accessql_src_completo.zip` (todo o `src/`, sem `Cargo.toml`). Descoberto o motivo do Ctrl+' fazer zoom: ver regra acima.
- 2026-11-07 c: passo 6b-1 testado (copiar, colar, cortar, registo inteiro, Ctrl+' na grelha e na edição). Notas do utilizador: colar na linha `(Novo)` não respeitava a numeração automática, colar um registo copiado não fazia nada e colar da folha de cálculo só trazia o 1.º valor. Corrigido em `area_trabalho.rs` (`colar_texto`, `recarregar_local`). Entregue de novo `accessql_src_completo.zip`.
- 2026-11-07 d: criado `CHECKLIST.md` (linha orientadora do projeto; o `PROJETO.md` continua a ser o contexto de cada sessão). Passo 7a escrito (sem compilar): `desfazer.rs` (novo), `db.rs` (`valor_bruto`, `registo_bruto`, `definir_celula`, `inserir_bruto`, `inserir_linhas_ids`), `area_trabalho.rs` (histórico em `gravar_celula`, `eliminar_registo`, colagem; Ctrl+Z/Y; `clip_acao`), `ribbon.rs` (QAT e Área de Transferência), `estado.rs`, `ficheiro.rs`, `consulta.rs`, `estrutura.rs`, `renomear.rs`, `csvio.rs`, `main.rs`. Inclui o pacote 6b-1b da colagem de registos.
- 2026-11-07 e: passo 7a testado (tudo bem). Passo 7b escrito (sem compilar): `totais.rs` e `indices.rs` (novos), `db.rs` (`Indice`, `lista_indices`, `criar_indice`, `eliminar_indice`), `ribbon.rs` (Registos: Totais; retirados Ortografia, Mais, Formatação de Texto, Macro, Documentador, Analisador, Mover Dados e os botões decorativos da Folha de Dados), `estrutura.rs` (`campo_na_estrutura`, `foco_novo`), `estado.rs`, `app.rs`, `ficheiro.rs`, `main.rs`.
- 2026-11-07 f: passo 7b dado como testado (utilizador pediu para continuar). Passo 7c escrito (sem compilar): `relacoes.rs` (novo), `db.rs` (`Relacao`, `Campo.ref_tabela/ref_coluna`, `lista_relacoes`, `verificar_relacoes`, `opcoes_fk`, `foreign_keys_ativas`, `definir_foreign_keys`, `traduz` público, REFERENCES nas definições, `foreign_key_list` em `info_tabela`), `estrutura.rs` (coluna «Refere a», `combo_fk`), `ribbon.rs` (botão Relações), `area_trabalho.rs` (erros traduzidos), `estado.rs`, `app.rs`, `ficheiro.rs`, `main.rs`. Escolhida a integridade referencial ligada por defeito (o utilizador não respondeu à pergunta).
- 2026-11-07 g: passos 7b e 7c testados (tudo bem). Passo 7d escrito (sem compilar): `eliminar_obj.rs` (novo), `db.rs` (`contar_registos`, `consultas_que_mencionam`, `eliminar_objeto`), `navegacao.rs` (menu «Eliminar (Delete)»), `app.rs` (Delete no painel), `estado.rs`, `ficheiro.rs`, `main.rs`. Por fazer a seguir: validação na folha de dados (em curso a análise: `TableData.tipos/obrig`, `validacao.rs`, `gravar_celula` com `reabrir`).
- 2026-11-19: auditoria completa do working tree e melhoramentos. **Correções:** `guardar_copia` passou a gravar num ficheiro temporário e só depois substituir o destino (antes apagava o destino primeiro); `guardar_vista` verifica que a instrução é mesmo SELECT/WITH; mensagem de eliminação concorda com tabela/consulta e a seleção é reposta; `.gitignore` desduplicado. **Validação:** novo `validacao.rs` com testes (inteiros, reais, Sim/Não, datas com anos bissextos, obrigatório), metadados `TableData.tipos/obrig` obtidos por `PRAGMA table_info`, aplicados em `gravar_celula`, colagem e importação CSV (colagem/importação validam tudo antes de gravar). **Limpeza:** removidos menus pendentes (`menus.rs`, `dd_items`, campo `dropdown`, `act`/`rb`/`stack`), botões Ajuda e Guardar, cartão inativo ligado a Novo, texto «Clique para Adicionar», ícones/variáveis mortas e `id_source` → `id_salt`. `cargo check` sem avisos; `cargo test`: 6/6. Ficheiros alterados: `main.rs`, `db.rs`, `validacao.rs` (novo), `area_trabalho.rs`, `csvio.rs`, `ficheiro.rs`, `eliminar_obj.rs`, `consulta.rs`, `estrutura.rs`, `indices.rs`, `relacoes.rs`, `estado.rs`, `app.rs`, `ribbon.rs`, `navegacao.rs`, `backstage.rs`, `icones.rs`, `widgets.rs`, `filtros.rs`, `dados_exemplo.rs`, `.gitignore`, `PROJETO.md`, `CHECKLIST.md`; eliminado `menus.rs`. Por fazer: testes manuais na interface e o resto da checklist.
- 2026-11-19 b: utilizador confirmou os testes na interface. Commit local de todo o trabalho pendente (passos 7a–7d + melhoramentos), identidade e licença: novo `README.md` (descrição, compilação FreeBSD, não afiliação, avisos das dependências), `LICENSE` com o texto oficial da GPL-3.0-or-later, `.gitignore` com resíduos do Hugo. Removidos `.hugo_build.lock` e `public/`. Ficheiros alterados: `README.md`, `LICENSE` (novos), `.gitignore`, `PROJETO.md`, `CHECKLIST.md`. Por fazer: ícone próprio, nome mais distinto, build release.
- 2026-11-19 c: "processa tudo". Push de `main` e da tag `passo7`. **Ícone:** novo `Icon::Logo` (cilindro de base de dados azul, desenhado em código) na barra de acesso rápido; nome mantido como AccesSQL (decisão registada na secção 7). **Campos existentes:** `Database::alterar_campo` em `db.rs` recria a tabela numa transação (copia dados, recria índices/gatilhos, preserva ações das relações, nome e vistas; desliga e volta a ligar `foreign_keys`); novos helpers `acoes_fk`/`objetos_da_tabela`, tradução de erros `NOT NULL`/`UNIQUE`, `em_memoria()` para testes; na Vista Estrutura, botão «Alterar» abre tipo/obrigatório/omissão com aplicação explícita. **Demo removida:** `Database::demo()`, `abrir_demo()`, cartão do backstage, `dados_exemplo.rs` e o `mod`. `cargo check` sem avisos; `cargo test`: 9/9. Ficheiros alterados: `db.rs`, `estrutura.rs`, `icones.rs`, `ribbon.rs`, `ficheiro.rs`, `backstage.rs`, `main.rs`, `PROJETO.md`, `CHECKLIST.md`; eliminado `dados_exemplo.rs`. Build release concluído (`target/release/accessql`, FreeBSD x86-64). Por fazer: teste manual na interface e commit/push finais.
- 2026-11-19 d: **zoom global** em `app.rs`: Ctrl + roda do rato ajusta `ctx.zoom_factor()` entre 0,5 e 2,5 (os atalhos de teclado do egui continuam desativados). **Passo 9** em `db.rs`: `Campo` ganhou `unico`, `check` e `autoincrement`; `definicao` emite `AUTOINCREMENT`, `UNIQUE` e `CHECK`; `validar_campo` valida a expressão CHECK e a regra do AUTOINCREMENT; novos helpers `segmentos_da_tabela`, `primeiro_ident`, `dequote_ident`, `palavra_no_inicio`, `sem_constraint`, `contem_palavra_fora`, `extrair_check`, `aplicar_restricoes_inline` e `restricoes_tabela` leem o `CREATE TABLE` original; `info_tabela` completa os campos com o que o catálogo não devolve; `criar_tabela_com_restricoes` aceita restrições de tabela; `acrescentar_campo` aceita CHECK mas recusa UNIQUE/AUTOINCREMENT; `alterar_campo(tabela, campo, &Campo)` preserva restrições inline e de tabela além de dados, índices, gatilhos e relações; `traduz` cobre CHECK/AUTOINCREMENT. Em `estrutura.rs`: colunas «Único», «Verificação (CHECK)» e «Auto» na grelha (nova e existente), controlos no painel «Alterar» e no campo a acrescentar, painel «Restrições de tabela» (editável só em tabela nova) e atualização das restrições após cada operação. `cargo check --locked` sem avisos; `cargo test --locked`: 13/13. Ficheiros alterados: `src/app.rs`, `src/db.rs`, `src/estrutura.rs`, `PROJETO.md`, `CHECKLIST.md`. Teste manual confirmado pelo utilizador; build release concluído; commit/push com tag `passo9`.
