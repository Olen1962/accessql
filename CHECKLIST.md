# AccesSQL: lista de funcionalidades

Legenda: `[x]` feito · `[ ]` por fazer · «(escrito, por testar)» = código entregue, ainda sem confirmação na interface.
Atualizada em 2026-11-19.

- **Ficheiro (backstage)**
  - [x] Novo (base de dados vazia `.accsql`, escolha de nome e pasta)
  - [x] Abrir (recusa ficheiros que não sejam SQLite, com mensagem clara)
  - [x] Recentes (lista guardada em `~/.config/accessql/recentes.txt`)
  - [x] Informações (nome, caminho, tamanho, tabelas, consultas, versão do SQLite)
  - [x] Compactar e Reparar (`integrity_check` + `VACUUM`)
  - [x] Guardar Cópia Como (`VACUUM INTO`)
  - [x] Fechar Base de Dados
  - [x] Sair
  - [x] Gravação automática (não há «Guardar»: cada alteração fica logo gravada)
  - [x] Base de demonstração removida: não era funcional (dados em memória que não ficavam guardados)
  - [x] Removidos `Database::demo()`, `abrir_demo()`, o cartão do backstage e `dados_exemplo.rs`
  - [ ] Guardar Script / Abrir Script (`.sql`) (ideia, a decidir)

- **Editar e teclado**
  - [x] Copiar (Ctrl+C) uma célula ou um registo inteiro (separado por tabuladores)
  - [x] Cortar (Ctrl+X)
  - [x] Colar (Ctrl+V) numa célula
  - [x] Colar vários campos e vários registos (da grelha ou de uma folha de cálculo), com numeração automática
  - [x] Ctrl+' copia o valor da célula de cima (na grelha e durante a edição)
  - [x] F2 universal (célula, nome do campo, nome de tabela ou consulta)
  - [x] Delete limpa a célula; Backspace abre a edição vazia
  - [x] Setas, Ctrl+setas, Home, End, PgUp, PgDn
  - [x] Backspace e setas nas caixas de texto (correção do método de entrada do X11)
  - [x] Zoom do teclado do egui desativado (conflitava com o apóstrofo em teclados portugueses)
  - [x] Ctrl + roda do rato aumenta/diminui a escala de toda a interface (0,5× a 2,5×, em passos de 5%)
  - [ ] Confirmar acentos (ç, ã, é) depois da correção do teclado
  - [x] Desfazer e Refazer: Ctrl+Z, Ctrl+Y (ou Ctrl+Shift+Z) e botões da barra de acesso rápido; cobrem células, registos criados e eliminados, colagens
  - [ ] Ctrl+; (data de hoje) e Ctrl+Shift+; (hora) (sugestão)
  - [ ] Substituir
  - [ ] Ir Para (primeiro, anterior, seguinte, último, novo)
  - [ ] Selecionar

- **Ribbon: Base**
  - [x] Vista (alterna dados e estrutura; nas consultas, dados e SQL)
  - [x] Ordenar e Filtrar: Filtro, Ascendente, Descendente, Remover Ordenação, Seleção, Remover Filtros, Alternar Filtro
  - [x] Registos: Novo, Eliminar (com confirmação), Atualizar Tudo
  - [x] Localizar (foca a caixa «Procurar»)
  - [x] Botões Cortar e Copiar do ribbon ligados
  - [x] «Colar» do ribbon retirado: o egui não deixa ler a área de transferência, só funciona com Ctrl+V
  - [ ] Ordenar e Filtrar: Avançadas
  - [x] Registos: «Totais» ligado (janela com contagem, soma, média, mínimo e máximo da coluna selecionada); Ortografia e Mais retirados
  - [x] Pincel de Formatos retirado
  - [x] Grupo Formatação de Texto retirado

- **Ribbon: Criar**
  - [x] Estrutura da Tabela (desenhador de tabelas novas)
  - [x] Tabela (SQL) (abre o modelo `CREATE TABLE`)
  - [x] Consulta SQL
  - [x] Grupos sem função retirados (Modelos, Formulários, Relatórios, Macros)

- **Ribbon: Dados Externos**
  - [x] Importar CSV
  - [x] Exportar CSV
  - [x] Grupos sem função retirados (Excel, Outlook, XML, HTML, Word)

- **Ribbon: Ferramentas de Bases de Dados**
  - [x] Compactar e Reparar
  - [x] Relações: janela com as chaves estrangeiras (tabela, campo, a que se refere, ao eliminar, ao atualizar), interruptor da integridade referencial e «Verificar integridade»
  - [x] Índices: janela para ver, criar (únicos ou não, vários campos) e eliminar os índices de uma tabela
  - [x] Botões sem função retirados (Macro, Documentador, Analisador, Mover Dados)

- **Ribbon: Folha de Dados (separador contextual)**
  - [x] Novo Campo, Renomear Campo e Eliminar Campo abrem a Vista Estrutura já posicionada no campo selecionado; botões decorativos retirados
  - [x] Tipo de Dados: combo na Vista Estrutura (criar e alterar campos)
  - [x] Validação: tipos e `NOT NULL` ao gravar/colar/importar; `UNIQUE`, `CHECK` e `DEFAULT` tratados pelo SQLite (os três também se definem na estrutura; mensagens de erro traduzidas)

- **Barra de acesso rápido e janela**
  - [x] Título e identidade AccesSQL na janela
  - [x] Aspeto Office 2007–2010 mantido
  - [x] Botões Desfazer e Refazer ligados; «Guardar» retirado (a gravação é automática)
  - [x] Botão de ajuda «?» removido (botões sem função real não ficam)

- **Painel de navegação**
  - [x] Tabelas e Consultas lidas do catálogo da base de dados
  - [x] Caixa «Procurar...» a filtrar a lista
  - [x] Abrir por duplo clique
  - [x] Menu de contexto: Abrir, Vista Estrutura, Vista SQL, Mudar o Nome (F2)
  - [x] Um só separador por objeto (a mudança de vista troca o modo no mesmo separador)
  - [x] Eliminar tabela ou consulta (Delete ou botão direito), com confirmação; recusa eliminar tabelas referidas por relações; avisa das consultas que as usam (implementado; falta teste manual)
  - [ ] Duplicar tabela ou consulta
  - [ ] Agrupar ou ocultar objetos

- **Folha de dados**
  - [x] Ver tabelas e consultas
  - [x] Editar células (duplo clique, F2 ou escrever)
  - [x] Linha `(Novo)` com numeração automática
  - [x] Seletor de registo (coluna cinzenta)
  - [x] Eliminar registo (Delete, botão direito ou ribbon), com confirmação
  - [x] Barra de registos (navegação e «N de M»)
  - [x] Só as linhas visíveis são desenhadas; cabeçalho fixo
  - [x] Ordenar (seta do cabeçalho, ribbon, menu) com indicador ▲ ▼
  - [x] Filtrar: igual, diferente, contém, não contém; remover; alternar
  - [x] Filtrar por seleção e excluir seleção (botão direito)
  - [x] Procurar com realce das células encontradas
  - [x] Validação do valor ao gravar (tipo do campo, obrigatório, datas, Sim/Não); também na colagem e na importação CSV
  - [ ] Largura das colunas ajustável com o rato
  - [ ] Linha de totais
  - [x] Totais da coluna numa janela
  - [ ] Ordenar por várias colunas

- **Vista Estrutura**
  - [x] Ver os campos (tipo, obrigatório, valor por omissão, chave primária)
  - [x] Ver a definição SQL original
  - [x] Criar tabelas novas
  - [x] Acrescentar, renomear e eliminar campos
  - [x] Mudar o tipo, a obrigatoriedade ou o valor por omissão de um campo existente: botão «Alterar» na Vista Estrutura; recriação da tabela numa transação, com cópia dos dados, índices, gatilhos, relações e vistas preservados (3 testes unitários)
  - [x] `UNIQUE`/`CHECK` por campo e `AUTOINCREMENT` ao criar tabelas; restrições de tabela `UNIQUE (...)`/`CHECK (...)` numa caixa de texto
  - [x] Preservação de `UNIQUE`/`CHECK`/`AUTOINCREMENT` (por campo e de tabela) ao alterar campos existentes (4 testes unitários novos)
  - [x] Indicar chaves estrangeiras (coluna «Refere a») ao criar tabelas e ao acrescentar um campo; ver as que já existem
  - [x] Criar e eliminar índices (pela janela Índices, em Ferramentas de Bases de Dados; implementado; falta teste manual)

- **Vista SQL e consultas**
  - [x] Editor com F5 ou Ctrl+Enter
  - [x] Scripts com várias instruções
  - [x] Resultados em grelha (máximo 100 000 linhas), erros por instrução
  - [x] Guardar como Consulta (`VIEW`), reabrir, editar e substituir
  - [x] Mudar o nome de uma consulta
  - [x] Exportar o resultado (pelo Exportar CSV)
  - [ ] Realce de sintaxe
  - [ ] Histórico de consultas
  - [ ] Consultas de ação guardadas (o SQLite não as suporta; seria por scripts)

- **Importar e exportar CSV**
  - [x] Deteção do separador (`;` `,` tabulador `|`) e da codificação (UTF-8 ou Windows-1252)
  - [x] Pré-visualização antes de importar
  - [x] Importar para tabela nova (tipos INTEGER, REAL, TEXT inferidos)
  - [x] Importar para tabela existente (por nome do cabeçalho ou por posição)
  - [x] Importação numa só transação (se uma linha falhar, nada é gravado)
  - [x] Exportar o que está no separador, com ordenação e filtros
  - [x] Opções: separador, cabeçalho, UTF-8 com BOM para o Excel

- **Projeto, código e git**
  - [x] Repositório no GitHub (público) com histórico limpo
  - [x] `PROJETO.md` para retomar sessões
  - [x] `.gitignore` (`target/`, `*.accsql`)
  - [x] Tags `passo1` a `passo6`
  - [x] Etiquetar o `passo6b` e o `passo7a`
  - [x] Limpar os avisos do compilador (`cargo check` sem avisos; `id_source` → `id_salt`, imports e código morto removidos)
  - [x] Passo 7: limpar o ribbon e os menus (retirar tudo o que não tenha função)
  - [x] Testes automáticos: 5 de validação e 8 em `db.rs` (metadados, alteração de campos, nulos, relações, UNIQUE/CHECK/AUTOINCREMENT e parser) — `cargo test`: 13/13

- **Identidade, licença e distribuição**
  - [x] Ícone próprio no lugar do «A» vermelho: cilindro de base de dados azul desenhado em código (`Icon::Logo` na barra de acesso rápido)
  - [x] Nome mantido como «AccesSQL» por decisão: está consolidado no repositório e na configuração; a distinção face ao Access faz-se pelo ícone e pela nota de não afiliação no README
  - [x] Licença GPL-3.0-or-later (`LICENSE`, texto oficial)
  - [x] `README.md` com descrição, compilação na FreeBSD e nota de não afiliação à Microsoft
  - [x] Lista dos avisos de direitos de autor das bibliotecas (no README)
  - [x] Compilação em modo release verificada (`cargo build --release --locked`; binário FreeBSD x86-64 em `target/release/accessql`)

- **Fora do âmbito desta versão**
  - Formulários e Relatórios
  - Macros e código
  - Importação de Excel, Outlook, XML, HTML e Word
  - Servidores de bases de dados (só SQLite local)
  - Utilizadores, contas e ligação à rede
