# PROJETO: AccesSQL

> Documento de contexto permanente do projeto. Colar no início de cada
> sessão. As tarefas, estados de conclusão, testes e pendências
> detalhadas pertencem ao `CHECKLIST.md`. No fim de cada sessão,
> atualizar apenas o estado resumido e o registo de sessões deste
> ficheiro, sem duplicar a checklist.

## 1. O que é

**AccesSQL** é uma aplicação desktop em Rust (egui/eframe) para gerir
bases de dados SQLite locais, com uma interface inspirada no Microsoft
Access 2007--2010: ribbon, painel de navegação, folha de dados e
backstage «Ficheiro».

-   O projeto começou como um modelo visual (mock); a interface mantém a
    identidade visual, mas as operações são reais.
-   Aplicação local e offline: sem início de sessão, rede ou pesquisas
    online.
-   Não usar «Microsoft Access» como nome ou marca em títulos ou textos
    da interface. O nome do produto é **AccesSQL**.

## 2. Como trabalhar comigo

-   Responder em **português europeu**.
-   O assistente não deve presumir que consegue consultar o repositório
    GitHub: o utilizador fornece os ficheiros relevantes.
-   O assistente não compila nem executa a aplicação. O utilizador corre
    `cargo check`, `cargo test`, `cargo run` ou `cargo build --release`
    e fornece os resultados.
-   Devolver **ficheiros completos** dos módulos alterados, com o
    respetivo caminho (por exemplo, `src/db.rs`), e não apenas excertos.
-   Se for necessário um ficheiro que ainda não foi fornecido, pedi-lo
    antes de propor alterações.
-   Preservar o aspeto Office 2010 existente (`tema.rs`, `widgets.rs`,
    `icones.rs`). Não redesenhar a interface sem pedido.
-   Um botão sem função real deve ser removido, não mantido apenas como
    elemento decorativo.
-   No fim de cada sessão, indicar os ficheiros alterados e fornecer um
    resumo curto para o registo de sessões.
-   Ser prático e conciso: privilegiar alterações concretas e código em
    vez de explicações extensas.

## 3. Ambiente e stack

-   Sistema de desenvolvimento: **FreeBSD (X11)**.
-   Repositório: `https://github.com/Olen1962/accessql`, público, ramo
    `main`, acesso remoto por SSH através de `gh`.
-   Rust edition 2021.
-   `eframe = 0.29`, com `default-features = false` e as features
    `default_fonts`, `glow`, `x11`.
-   `rfd = 0.15` para diálogos de ficheiros.
-   `rusqlite = { version = "0.32", features = ["bundled"] }`.
-   Pacote/binário: `accessql`.
-   Extensão dos ficheiros de bases de dados: `.accsql`.

### Particularidades de FreeBSD/X11

-   Para `sed` com edição no local, usar `sed -i ''`.
-   No fim de `update()`, manter `ctx.output_mut(|o| o.ime = None);`.
    Esta correção evita que o método de entrada XIM consuma Backspace e
    as setas nas caixas de texto. Depois de alterações relacionadas com
    teclado, testar também caracteres acentuados (`ç`, `ã`).
-   O egui tinha atalhos de zoom por teclado que interferiam com Ctrl+'
    em teclados portugueses. Manter
    `ctx.options_mut(|o| o.zoom_with_keyboard = false)` e aceitar
    `Key::Quote` ou `Key::Minus` com Ctrl para copiar o valor da célula
    de cima.
-   O zoom global é feito com **Ctrl + roda do rato**. O fator está
    limitado a 0,5×--2,5×. O sentido atual é: roda para baixo =
    aumentar; roda para cima = diminuir.
-   A grelha e o ribbon são desenhados manualmente com `painter`; não
    introduzir `egui_extras` para os desenhar.
-   O `.gitignore` deve incluir `target/`, porque o executável de
    desenvolvimento ultrapassa o limite de tamanho aceite pelo GitHub.
-   Depois de aplicar um pacote ZIP com os módulos Rust, executar
    `touch src/*.rs` antes de compilar, para evitar que diferenças entre
    relógios levem o Cargo a considerar os ficheiros atualizados.
    Confirmar que a saída mostra `Compiling accessql`.
-   Antes de aplicar pacotes, verificar `git status` e evitar misturar
    ficheiros de passos diferentes. `accessql_src_completo.zip`, quando
    fornecido, contém todos os `.rs` de `src/`, mas não inclui
    `Cargo.toml`.

## 4. Estrutura do código (`src/`)

  -----------------------------------------------------------------------
  Ficheiro                            Responsabilidade
  ----------------------------------- -----------------------------------
  `main.rs`                           Janela eframe e declaração dos
                                      módulos

  `app.rs`                            Ciclo `update()`, QAT, separadores,
                                      barra de estado, backstage e
                                      composição da interface

  `estado.rs`                         Estado global `AccessApp`,
                                      `ObjKind`, `Doc`, edição, navegação
                                      e abertura de documentos

  `ribbon.rs`                         Barra de título, QAT, separadores e
                                      grupos do ribbon

  `navegacao.rs`                      Painel de navegação e menus de
                                      contexto

  `area_trabalho.rs`                  Separadores e folha de dados:
                                      edição, registo `(Novo)`,
                                      eliminação, atualização e linhas
                                      visíveis

  `backstage.rs`                      Backstage «Ficheiro»: informações,
                                      recentes, novo e operações de
                                      ficheiro

  `db.rs`                             Camada SQLite: abertura/criação,
                                      catálogo, leitura, escrita,
                                      consultas, DDL, índices, gatilhos,
                                      relações e operações de estrutura

  `ficheiro.rs`                       Operações de ficheiro de
                                      `AccessApp`: novo, abrir, guardar
                                      cópia, fechar e compactar

  `recentes.rs`                       Lista de bases de dados recentes em
                                      `~/.config/accessql/recentes.txt`

  `consulta.rs`                       Vista SQL, execução de instruções,
                                      resultados e consultas guardadas
                                      como `VIEW`

  `estrutura.rs`                      Vista Estrutura: criação de tabelas
                                      e gestão de campos e restrições

  `csvio.rs`                          Importação e exportação de CSV

  `filtros.rs`                        Ordenação, filtros e procura na
                                      folha de dados

  `validacao.rs`                      Validação dos valores antes da
                                      gravação e testes unitários

  `desfazer.rs`                       Histórico de desfazer/refazer

  `totais.rs`                         Janela «Totais»

  `indices.rs`                        Janela «Índices»

  `relacoes.rs`                       Janela «Relações» e integridade
                                      referencial

  `eliminar_obj.rs`                   Eliminação de tabelas e consultas

  `renomear.rs`                       Mudança do nome de tabelas e
                                      consultas

  `duplicar_obj.rs`                   Duplicação de tabelas e consultas

  `status.rs`                         Barra de estado

  `tema.rs`                           Paleta e constantes de cor

  `widgets.rs`                        Botões e grupos do ribbon

  `icones.rs`                         Ícones desenhados em código,
                                      incluindo `Icon::Logo`
  -----------------------------------------------------------------------

A lista acima descreve a organização conhecida do código. Se a estrutura
real mudar, atualizar esta secção com base nos ficheiros fornecidos pelo
utilizador.

## 5. Decisões de arquitetura e comportamento

### 5.1. Base de dados e objetos

-   Motor: **SQLite**, através de `rusqlite`. Cada base de dados é um
    ficheiro SQLite normal.
-   Extensão: **`.accsql`**. O conteúdo é SQLite; não usar `.accdb`.
-   O painel de navegação apresenta **Tabelas** e **Consultas**.
-   As consultas guardadas são `VIEW`s do SQLite.
-   Formulários e Relatórios estão fora da primeira versão. `ObjKind`
    não deve incluir `Form` ou `Report`, nem devem existir vistas
    `form_view`/`report_view`.
-   Vistas disponíveis: Folha de Dados, Estrutura e SQL.

### 5.2. Ficheiros, arranque e gravação

-   Ao arrancar, não há base de dados aberta. O backstage abre em
    «Recente» ou em «Novo», se não houver bases recentes.
-   Os recentes ficam em `~/.config/accessql/recentes.txt`, com um
    máximo de 10 entradas.
-   A abertura recusa ficheiros que não sejam bases SQLite.
-   Não existe botão «Guardar»: o SQLite grava cada alteração
    imediatamente.
-   «Guardar Cópia Como» usa `VACUUM INTO`.
-   «Compactar e Reparar» combina `VACUUM` com `integrity_check`.

### 5.3. Separadores e mudança de vista

-   Cada objeto (tabela ou consulta) tem **um único separador**.
-   O botão «Vista» e o menu de contexto mudam o modo no mesmo
    separador:
    -   tabela: Folha de Dados ↔ Estrutura;
    -   consulta guardada: dados ↔ SQL.
-   Abrir uma tabela já aberta ativa o separador existente e apresenta a
    Folha de Dados.
-   Se uma consulta tiver alterações SQL por guardar, não se muda de
    vista sem tratar primeiro essas alterações.
-   As consultas e os resultados SQL são só de leitura e não têm linha
    `(Novo)`.

### 5.4. Folha de dados e teclado

-   O seletor à esquerda seleciona o registo. Delete elimina o registo
    após confirmação.
-   Com uma célula selecionada, Delete limpa a célula; Backspace, F2,
    Enter ou escrever inicia a edição.
-   Ctrl+setas navega para o primeiro/último registo ou campo.
-   Ctrl+C copia uma célula ou, se o seletor de registo estiver
    selecionado, o registo inteiro em texto separado por tabulações.
-   Ctrl+X copia e limpa.
-   Ctrl+V cola. Na linha `(Novo)`, cada linha de texto pode criar um
    registo; ao colar um registo copiado da própria grelha, o
    identificador INTEGER PRIMARY KEY é gerado de novo. Nos registos
    existentes, os valores são colocados a partir da célula selecionada;
    linhas excedentes são ignoradas.
-   Ctrl+' copia o valor da mesma coluna no registo imediatamente acima.
    Em teclados portugueses, o código também considera Ctrl+Minus.
-   Delete na célula significa limpar o valor (NULL), não eliminar o
    registo. Para eliminar um registo, usar o seletor de registo.

### 5.5. Desfazer e refazer

-   Cada gravação é registada em `hist_pend`; `fechar_passo` agrupa as
    alterações de uma operação num único passo (por exemplo, uma colagem
    extensa).
-   Guardam-se os valores brutos do SQLite, incluindo BLOB e NULL.
-   O histórico tem um máximo de 200 passos.
-   O histórico é limpo ao fechar/abrir a base de dados, executar SQL
    que altere dados, alterar a estrutura, mudar o nome de objetos ou
    importar CSV.
-   Não existe botão «Colar» no ribbon: a aplicação recebe o texto da
    área de transferência através de Ctrl+V.

### 5.6. Ordenação, filtros e procura

-   Ordenação e filtros pertencem a cada separador (`Doc.ordem`,
    `Doc.filtros`, `filtro_ativo`).
-   A ordenação usa `ORDER BY` e `COLLATE NOCASE`.
-   Os filtros acumulam-se com AND e incluem igual, diferente, contém e
    não contém. O filtro vazio corresponde a NULL ou cadeia vazia.
-   O menu da coluna abre pela seta do cabeçalho, pelo botão direito no
    cabeçalho ou pelo botão «Filtro» do ribbon.
-   O cabeçalho assinala a coluna ordenada com ▲/▼ e as colunas
    filtradas a laranja.
-   «Procurar» pesquisa em memória nas linhas carregadas, realça os
    resultados e usa Enter para avançar para a ocorrência seguinte.

### 5.7. Validação

-   Antes de `UPDATE` ou `INSERT`, a aplicação valida o tipo declarado e
    a obrigatoriedade da coluna, com base em `PRAGMA table_info` e nos
    metadados `TableData.tipos`/`obrig`.
-   Inteiros, reais (ponto ou vírgula decimal), Sim/Não e datas
    (`AAAA-MM-DD` ou `DD/MM/AAAA`) exigem formato válido. As datas são
    verificadas, incluindo anos bissextos.
-   `TEXT`, `BLOB` e tipos personalizados aceitam texto livre.
-   A validação aplica-se à edição de células, colagem e importação CSV.
    Uma colagem ou importação inválida não grava parcialmente os dados.
-   `UNIQUE`, `CHECK` e chaves estrangeiras continuam a ser validados
    pelo SQLite; as mensagens relevantes são traduzidas para português.

### 5.8. CSV

-   Na importação, detetam-se separadores `;`, `,`, tabulação ou `|` e
    codificação UTF-8 ou Windows-1252.
-   Ao criar uma tabela nova, inferem-se INTEGER, REAL ou TEXT por
    coluna. Valores com zeros à esquerda são tratados como texto; é
    aceite vírgula decimal.
-   Ao acrescentar dados a uma tabela existente, as colunas são
    associadas pelo nome do cabeçalho ou pela posição, se não houver
    cabeçalho.
-   A importação é uma única transação: se uma linha falhar, nada é
    gravado.
-   A exportação usa os dados apresentados no separador, incluindo
    ordenação e filtros; grava UTF-8, com BOM opcional e linhas CRLF.

### 5.9. Estrutura das tabelas e restrições

-   A Vista Estrutura permite criar tabelas com nome, tipo,
    obrigatoriedade, valor por omissão, chave primária, UNIQUE, CHECK e
    AUTOINCREMENT.
-   Uma chave primária INTEGER simples com «Auto» gera
    `INTEGER PRIMARY KEY AUTOINCREMENT`; chaves múltiplas geram
    `PRIMARY KEY (a, b)`.
-   Há um painel «Restrições de tabela» para definir restrições
    `UNIQUE (...)` e `CHECK (...)` na criação de tabelas.
-   Em tabelas existentes, é possível renomear, alterar e eliminar
    campos, bem como acrescentar campos.
-   Ao acrescentar um campo, um campo obrigatório exige valor por
    omissão. A operação aceita CHECK, mas não permite acrescentar chave
    primária, UNIQUE ou AUTOINCREMENT através de
    `ALTER TABLE ADD COLUMN`; para criar um índice único existe a janela
    «Índices».
-   Alterar um campo recria a tabela dentro de uma transação. A operação
    preserva dados, índices, gatilhos, relações e ações
    `ON DELETE`/`ON UPDATE`, nome da tabela, vistas, restrições inline e
    restrições de tabela UNIQUE/CHECK. Se algo falhar, a alteração é
    revertida.
-   O texto original do `CREATE TABLE` é analisado para recuperar
    restrições que o catálogo SQLite não devolve integralmente.
-   As restrições de tabela existentes são apresentadas para consulta,
    não para edição direta.

### 5.10. Chaves estrangeiras e relações

-   `PRAGMA foreign_keys = ON` é ativado ao abrir ou criar uma base de
    dados.
-   A janela «Relações» permite consultar as chaves estrangeiras,
    ativar/desativar a integridade referencial durante a sessão e
    verificar relações.
-   Ao desenhar uma tabela ou acrescentar um campo, «Refere a» permite
    escolher uma chave primária simples e gera
    `REFERENCES tabela(campo)`.
-   Não se acrescentam relações a campos já existentes pela interface;
    isso exigiria recriar a tabela.
-   A eliminação de uma tabela é bloqueada apenas quando essa tabela é
    referida por chaves estrangeiras de outras tabelas, mesmo que a
    integridade esteja desligada. A tabela que contém a relação pode ser
    eliminada.
-   A mensagem `FOREIGN KEY constraint failed` é traduzida.

### 5.11. Consultas SQL

-   Cada consulta SQL aberta tem o seu próprio separador («Consulta1»,
    etc.).
-   Scripts com várias instruções são executados em sequência, sem
    transação própria.
-   É apresentado o resultado da última instrução que devolva linhas,
    limitado a 100 000 linhas.
-   «Guardar como Consulta» cria uma `VIEW` e aceita apenas `SELECT`
    (incluindo formas válidas com `WITH`).
-   Se o separador já corresponder a uma consulta guardada, a gravação
    substitui a `VIEW` correspondente.
-   Consultas e resultados SQL não têm linha `(Novo)` e não são
    editáveis.

### 5.12. Renomear, duplicar e eliminar objetos

-   F2 no painel de navegação muda o nome da tabela ou consulta
    selecionada; na folha de dados edita a célula; na Vista Estrutura
    muda o nome do campo selecionado. O foco e o estado de edição
    determinam a ação.
-   Ctrl+D no painel de navegação, ou «Duplicar» no menu de contexto,
    abre um diálogo com o nome proposto `original_copia`.
-   A duplicação de uma tabela reescreve o DDL original, copia os dados,
    índices e gatilhos e mantém as restrições. É feita numa transação,
    com gestão temporária de `foreign_keys`; os nomes dos índices e
    gatilhos copiados são tornados únicos.
-   A duplicação de uma consulta copia o SQL da `VIEW`.
-   Se o nome escolhido já existir, nada é criado. A cópia fica
    selecionada e é aberta; o histórico de desfazer é limpo.
-   A eliminação pede confirmação com o nome e número de registos,
    informa que não pode ser anulada, fecha separadores do objeto e
    limpa o histórico.
-   A eliminação avisa sobre consultas que mencionam o objeto, mas não
    bloqueia por esse motivo.

## 6. Âmbito do ribbon

Manter apenas comandos com função real:

-   **Ficheiro:** Novo, Abrir, Guardar Cópia Como, Recentes, Fechar e
    Sair.
-   **Base:** Vistas (Folha de Dados, Estrutura e SQL), Cortar/Copiar,
    Ordenar e Filtrar, Registos (Novo, Eliminar, Atualizar e Totais) e
    Localizar. Não incluir Guardar, Pincel de Formatos, Formatação de
    Texto ou Ortografia.
-   **Criar:** Tabela, Estrutura da Tabela e Consulta SQL. Não incluir
    modelos, formulários, relatórios, macros ou código.
-   **Dados Externos:** Importar CSV e Exportar CSV.
-   **Ferramentas de Bases de Dados:** Relações, Índices e Compactar e
    Reparar.
-   **Folha de Dados** (separador contextual): Novo campo, Eliminar
    campo e Renomear. A validação de tipos e NOT NULL é automática; a
    alteração de tipos e restantes propriedades é feita na Vista
    Estrutura.

## 7. Identidade e distribuição

-   Título da janela: **AccesSQL**; identificador da aplicação:
    `with_app_id("accessql")`.
-   Barra de título: «Base1 : Base de Dados SQLite - AccesSQL».
-   Ícone próprio: cilindro de base de dados azul, desenhado em código
    como `Icon::Logo` em `icones.rs`. Não usar o «A» vermelho, imagens
    ou marcas do Access.
-   O nome **AccesSQL** está consolidado no repositório, no binário e em
    `~/.config/accessql`. Não o alterar sem pedido explícito.
-   O `README.md` descreve a aplicação, a compilação em FreeBSD e a não
    afiliação à Microsoft.
-   Licença: **GPL-3.0-or-later**, com o texto oficial em `LICENSE`.

## 8. Problemas conhecidos e critérios

-   Abertura dos diálogos `rfd` em FreeBSD deve continuar a ser
    verificada no ambiente real. Se falhar, considerar um campo de
    caminho no backstage.
-   Depois de alterações ao tratamento de texto/teclado, testar
    Backspace, setas e acentos no X11.
-   Não considerar uma alteração concluída só porque o código foi
    escrito: distinguir sempre «escrito», «compilado», «testado» e
    «commit/push».
-   Os problemas resolvidos e as tarefas ainda pendentes estão
    registados no `CHECKLIST.md`; não manter aqui uma segunda lista de
    estados.

## 9. Orientação para a próxima fase

O desenvolvimento funcional dos passos 1--10 está concluído segundo o
histórico fornecido. A continuação deve partir do `CHECKLIST.md`, que é
a fonte única para: - tarefas por fazer; - testes manuais e
automatizados; - estado de cada passo; - prioridades e sequência de
trabalho.

Antes de propor uma nova funcionalidade, confirmar primeiro a checklist
e os ficheiros atuais envolvidos. Não reabrir decisões já fechadas sem
uma razão técnica concreta ou pedido do utilizador.

## 10. Estado atual --- 2026-11-25

-   **Projeto:** AccesSQL, aplicação desktop Rust/egui para gestão local
    de bases SQLite.
-   **Último estado de repositório conhecido:** commit
    `a5ca12f82132ee4e56925f39f6abb45a1df84a20`, consultado em
    2026-11-19; passos 1--10 e tag `passo10` registados como concluídos.
-   **Funcionalidade:** o âmbito definido nas secções 5 e 6 está
    implementado, incluindo edição, estrutura, SQL, CSV, validação,
    relações, índices, desfazer/refazer, duplicação e zoom.
-   **Validação conhecida:** `cargo check --locked` sem avisos;
    `cargo test --locked` com 16/16 testes; build release concluído.
    Estes resultados pertencem à última sessão registada e não
    significam que tenham sido repetidos em 2026-11-25.
-   **Nesta atualização:** foi reorganizada a documentação para separar
    o contexto permanente (`PROJETO.md`) do acompanhamento de tarefas
    (`CHECKLIST.md`). Não foram alterados nem testados ficheiros de
    código nesta sessão.
-   **Fonte de verdade para o trabalho seguinte:** `CHECKLIST.md` para
    tarefas e progresso; ficheiros atuais fornecidos pelo utilizador
    para alterações de código.

## 11. Registo de sessões

Este registo resume marcos do projeto. A descrição detalhada das
tarefas, testes, ficheiros e pendências deve ficar no `CHECKLIST.md`,
evitando manter o mesmo histórico nos dois documentos.

-   **2026-11-05:** análise do mock, definição do âmbito e início da
    implementação SQLite.
-   **2026-11-05 a 2026-11-06:** implementação e validação dos passos
    iniciais: camada SQLite, folha de dados, operações de ficheiro,
    Vista SQL, correção do teclado XIM e Vista Estrutura.
-   **2026-11-06 a 2026-11-07:** separadores por objeto, F2, ordenação,
    filtros, procura, CSV, copiar/colar e desfazer/refazer.
-   **2026-11-07:** ribbon e menus simplificados; implementação de
    Totais, Índices, Relações e eliminação de objetos.
-   **2026-11-19:** auditoria e limpeza geral; validação de dados;
    identidade e licença; alteração de campos preservando restrições;
    remoção da base de demonstração; zoom global; preservação de
    UNIQUE/CHECK/AUTOINCREMENT; duplicação de tabelas e consultas.
    Testes, build release, commit/push e tags registados na checklist.
-   **2026-11-25:** atualização deste `PROJETO.md` para eliminar a
    duplicação com `CHECKLIST.md`. Sem alterações ao código e sem novos
    testes nesta sessão.
