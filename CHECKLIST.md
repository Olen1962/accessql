# AccesSQL: lista de funcionalidades

Legenda: `[x]` feito e testado · `[x]` por fazer · «(escrito, por testar)» = código entregue, ainda sem confirmação.
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
  - [x] Base de dados de demonstração (temporária, em memória)
  - [ ] Remover a base de demonstração e `dados_exemplo.rs`
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
  - [x] Zoom do egui desativado (conflitava com o apóstrofo)
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
  - [ ] Tipo de Dados
  - [ ] Validação (`NOT NULL`, `UNIQUE`, `CHECK`, `DEFAULT`)

- **Barra de acesso rápido e janela**
  - [x] Título e identidade AccesSQL na janela
  - [x] Aspeto Office 2007–2010 mantido
  - [x] Botões Desfazer e Refazer ligados; «Guardar» avisa que a gravação é automática
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
  - [ ] Mudar o tipo, a obrigatoriedade ou o valor por omissão de um campo existente (exige recriar a tabela)
  - [ ] Chaves primárias compostas e `UNIQUE`/`CHECK` ao criar tabelas
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
  - [x] Testes automáticos: 5 testes de validação e 1 teste de metadados em `db.rs` (`cargo test`: 6/6)

- **Identidade, licença e distribuição**
  - [ ] Ícone próprio no lugar do «A» vermelho (evita semelhança com o logótipo do Access)
  - [ ] Nome mais distinto do que «AccesSQL» (a decidir)
  - [x] Licença GPL-3.0-or-later (`LICENSE`, texto oficial)
  - [x] `README.md` com descrição, compilação na FreeBSD e nota de não afiliação à Microsoft
  - [x] Lista dos avisos de direitos de autor das bibliotecas (no README)
  - [ ] Compilação em modo release verificada (instruções já no README)

- **Fora do âmbito desta versão**
  - Formulários e Relatórios
  - Macros e código
  - Importação de Excel, Outlook, XML, HTML e Word
  - Servidores de bases de dados (só SQLite local)
  - Utilizadores, contas e ligação à rede
