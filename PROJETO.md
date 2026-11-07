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
| `db.rs` | Camada SQLite (`Database`, `TableData`): open/create/demo, catálogo, load, update/insert/delete, `VACUUM INTO`, compactar |
| `estrutura.rs` | Vista Estrutura: ver campos de uma tabela, acrescentar/renomear/eliminar campos, desenhar e criar tabelas novas; botão «Vista» que alterna folha de dados/estrutura |
| `consulta.rs` | Vista SQL: editor, executar (F5/Ctrl+Enter), resultados, guardar como `VIEW`, abrir `VIEW` para editar |
| `ficheiro.rs` | Novo, abrir, guardar cópia, fechar, compactar (métodos de `AccessApp`) |
| `recentes.rs` | Lista de recentes em `~/.config/accessql/recentes.txt` |
| `menus.rs` | Menus pendentes |
| `status.rs` | Barra de estado |
| `tema.rs` | Paleta e constantes de cor |
| `widgets.rs` | Botões grandes/pequenos, grupos do ribbon |
| `icones.rs` | Ícones desenhados em código |
| `dados_exemplo.rs` | Dados fixos do mock (**a eliminar**) |


## 5. Decisões tomadas

- Motor de dados: **SQLite** via `rusqlite`. Cada base de dados é um ficheiro SQLite normal.
- Extensão dos ficheiros: **`.accsql`** (SQLite por dentro; não usar `.accdb`, que o Access não abriria).
- Objetos do painel de navegação: **Tabelas** e **Consultas**. As consultas guardadas são **`VIEW`s** do SQLite.
- **Formulários e Relatórios ficam fora** da primeira versão (remover `Form`/`Report` de `ObjKind` e as vistas `form_view`/`report_view`).
- Vistas: Folha de Dados, Estrutura, **SQL** (editor + resultados).
- Vista Estrutura: tabela nova = grelha editável (nome, tipo, obrigatório, valor por omissão, chave primária) + «Criar Tabela» (uma chave primária INTEGER fica numerada automaticamente; várias chaves viram `PRIMARY KEY (a, b)`). Tabela existente = campos só de leitura com Renomear, Eliminar (confirmação) e «Acrescentar campo» (campo obrigatório exige valor por omissão; não se acrescentam chaves primárias). Mudar tipo/restrições de um campo existente NÃO é suportado (exigiria recriar a tabela).
- Vista SQL: um separador por consulta («Consulta1», …). Scripts com várias instruções são executados em sequência (sem transação própria); mostra-se o resultado da última que devolve linhas (máx. 100000). «Guardar como Consulta» cria uma `VIEW` (só `SELECT`); se o separador já veio de uma consulta guardada, substitui-a sem perguntar. Consultas e resultados SQL não têm linha `(Novo)` e não são editáveis.
- Folha de dados: clicar no seletor de registo (coluna esquerda) seleciona o registo; Delete elimina (com confirmação). Com uma célula selecionada, Delete limpa-a, Backspace ou F2/Enter/escrever editam. Ctrl+setas = primeiro/último registo e campo.
- Ao arrancar não há base de dados aberta e o backstage abre em «Recente» (ou «Novo», se não houver recentes).
- Recentes em `~/.config/accessql/recentes.txt` (máx. 10). Abrir recusa ficheiros que não sejam SQLite.
- «Guardar» não existe: o SQLite grava cada alteração logo. «Guardar Cópia Como» usa `VACUUM INTO`.
- Alterações menores são decididas dinamicamente à medida que o projeto avança.

## 6. Âmbito do ribbon (o que fica)

- **Ficheiro**: Novo, Abrir, Guardar Cópia Como, Recentes, Fechar, Sair.
- **Base**: Vistas (Folha de Dados / Estrutura / SQL), Cortar/Copiar/Colar, Ordenar e Filtrar (`ORDER BY` / `WHERE`), Registos (Novo, Guardar, Eliminar, Atualizar), Localizar. Saem: Pincel de Formatos, Formatação de Texto, Ortografia.
- **Criar**: Tabela, Estrutura da Tabela, Consulta SQL. Saem: Modelos, Partes da Aplicação, Formulários, Relatórios, Macros e Código.
- **Dados Externos**: Importar CSV, Exportar CSV. Saem: Excel, Outlook, dBASE, XML, HTML, Word.
- **Ferramentas de Bases de Dados**: Relações (chaves estrangeiras), Índices, Compactar e Reparar (`VACUUM` + `integrity_check`). Saem: Documentador, Analisador, Mover Dados.
- **Folha de Dados** (separador contextual): Novo campo, Eliminar campo, Renomear, Tipo de Dados, Validação (`NOT NULL`, `UNIQUE`, `CHECK`, `DEFAULT`).

## 7. Identidade (a aplicar)

- Título da janela: **AccesSQL**. `with_app_id("accessql")`.
- Barra de título: «Base1 : Base de Dados SQLite - AccesSQL».
- O ícone «A» vermelho mantém-se.

## 8. Problemas conhecidos

Resolvidos nos passos 1 e 2: dados fixos, `OBJECTS` constante, `Doc` sem dados, desenho de todas as linhas, pânico com 0 colunas/linhas, falta de edição.

Em aberto:

1. Os valores gravam-se como texto e o SQLite converte conforme a afinidade da coluna; não há validação até ao passo 5.
2. O diálogo de ficheiros (`rfd`) pode não abrir na FreeBSD; se falhar, acrescentar um campo de caminho no backstage.
3. Avisos de código não usado (ex.: `Icon::Form`, imports) e botões do ribbon ainda sem função: limpeza no passo 7.
4. A base de dados de demonstração (em memória, via `dados_exemplo.rs`) é temporária; sai no passo 7.

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
- [~] Passo 5: Vista Estrutura (Criar → Estrutura da Tabela; botão «Vista» no ribbon; menu de contexto «Vista Estrutura» nas tabelas; acrescentar/renomear/eliminar campos com `ALTER TABLE`): escrito, **por compilar**.
- [ ] Passos 6 e 7 (secção 9). Também por fazer: ordenar, filtrar e procurar na folha de dados (`ORDER BY` / `WHERE`).

**Próximo passo:** `touch src/*.rs` + `cargo run` com o pacote do passo 5 e testar (ver registo de sessões). Depois: ordenar/filtrar/procurar na folha de dados; mudar tipo/obrigatório/valor por omissão de um campo existente (exige recriar a tabela e copiar os dados, por isso fica à parte); validação na folha de dados.

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