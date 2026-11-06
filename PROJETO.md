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
- [~] Passo 3: Ficheiro → Novo / Abrir / Guardar Cópia Como / Fechar / Recentes / Compactar: escrito, **por compilar**.
- [ ] Passos 4 a 7 (secção 9). Antes do 4: ordenar, filtrar e procurar na folha de dados (`ORDER BY` / `WHERE`).

**Próximo passo:** `cargo check` com o pacote do passo 3 e testar: criar ficheiro, editar, fechar, reabrir pelos Recentes, ver que os dados ficaram guardados.

## 11. Registo de sessões

_(Acrescentar uma linha por sessão: data, o que foi feito, ficheiros alterados, o que ficou por fazer.)_

- 2026-11-05: análise do mock, definição de âmbito e decisões, criação deste ficheiro. Sem alterações ao código.
- 2026-11-05 b: passo 1 escrito (sem compilar). Novo `db.rs` (SQLite, base de demonstração em memória com `Clientes`/`Encomendas`/`Produtos` + 2 `VIEW`s); `estado.rs` (`NavGroup`, `Doc.data`, `refresh_nav`, `open_doc` carrega da BD; `ObjKind` só `Table`/`Query`); `navegacao.rs` lê o catálogo; `area_trabalho.rs` (datasheet lê `TableData`, removidos `form_view`/`report_view`, corrigido pânico com 0 colunas/0 linhas); `status.rs`; `ribbon.rs` (título AccesSQL); `main.rs`; `Cargo.toml` (nome `accessql`, `rusqlite`). Por fazer: `cargo check`, depois passo 2.
- 2026-11-05 c: passo 2 escrito (sem compilar). `db.rs` (rowid, update/insert/delete/fetch_row, `ID INTEGER PRIMARY KEY` na demo); `estado.rs` (`Edicao`, `scroll_sel`, `confirm_del`, `Doc.data` sem `Rc`); `area_trabalho.rs` (nova `datasheet` com edição, teclado, linhas visíveis, cabeçalho fixo; `gravar_celula`, `novo_registo`, `pedir_eliminar`, `dialogo_eliminar`, `recarregar_doc`); `ribbon.rs` (Registos ligado); `navegacao.rs` e `area_trabalho.rs` (ids dos campos de texto); `app.rs`. Por testar: edição, Tab/Enter, `(Novo)`, Eliminar.
- 2026-11-05 d: passos 1 e 2 compilados, testados e guardados no git (tags `passo1`, `passo2`). Passo 3 escrito (sem compilar): `db.rs` (`open`/`create`/`copy_to`/`compact`, erros em português), `ficheiro.rs` e `recentes.rs` (novos), `backstage.rs` reescrito (Informações reais, Recente, Novo com pasta e demo), `estado.rs` (`db_file`, `recentes`, `new_db_dir`; arranca sem BD e com o backstage aberto), `ribbon.rs` (título «AccesSQL» sem BD), `main.rs`. Por testar tudo isto.
