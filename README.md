# AccesSQL

Aplicação desktop para gerir bases de dados **SQLite locais**, com uma
interface inspirada no **Microsoft Access 2007--2010**: ribbon, painel
de navegação, folha de dados e backstage «Ficheiro».

Tudo é local e offline: não há contas, ligação à rede nem pesquisas
online. Cada base de dados é um ficheiro SQLite normal com a extensão
`.accsql`, pelo que pode ser aberto por qualquer ferramenta compatível
com SQLite.

> **Não afiliação:** o AccesSQL não está afiliado, endossado nem
> patrocinado pela Microsoft Corporation. «Microsoft Access» é uma marca
> registada da Microsoft Corporation. Este projeto não contém código,
> componentes ou dados do Microsoft Access. A semelhança visual
> destina-se apenas a proporcionar familiaridade a quem utiliza esse
> tipo de ferramenta.

## Funcionalidades

-   **Ficheiro:** criar, abrir, consultar recentes e informações da base
    de dados, compactar e verificar a integridade (`VACUUM` e
    `integrity_check`), guardar uma cópia e fechar. A gravação é
    automática; não existe comando «Guardar».
-   **Tabelas e consultas:** painel de navegação ligado ao catálogo
    SQLite. As consultas guardadas são `VIEW`s. Cada objeto tem um único
    separador, que alterna entre Folha de Dados, Estrutura e SQL,
    conforme o tipo de objeto.
-   **Folha de Dados:** visualizar e editar células, criar registos na
    linha `(Novo)`, eliminar registos, ordenar, filtrar e procurar com
    realce. Inclui totais da coluna e validação dos valores antes de
    gravar.
-   **Estrutura:** criar tabelas e acrescentar, renomear, alterar ou
    eliminar campos. Suporta chaves primárias, `NOT NULL`, valores por
    omissão, `UNIQUE`, `CHECK`, `AUTOINCREMENT` e chaves estrangeiras.
-   **Consultas SQL:** editor SQL, execução com `F5` ou `Ctrl+Enter`,
    scripts com várias instruções e resultados apresentados em grelha.
    Um `SELECT` pode ser guardado como consulta (`VIEW`).
-   **Dados externos:** importar e exportar CSV, com deteção do
    separador e da codificação. A importação pode criar uma tabela ou
    acrescentar registos a uma tabela existente.
-   **Ferramentas:** gestão de relações e índices, verificação da
    integridade referencial e eliminação de tabelas e consultas com
    confirmação.
-   **Duplicação:** duplicar tabelas, preservando dados, restrições,
    índices e gatilhos, ou duplicar consultas.
-   **Edição:** copiar, cortar e colar células, registos e múltiplas
    linhas; copiar o valor da célula imediatamente acima;
    Desfazer/Refazer.
-   **Interface:** ribbon e ícones próprios, atalhos de teclado e zoom
    global com `Ctrl` + roda do rato.

Fora do âmbito desta versão: formulários, relatórios, macros, importação
direta de Excel/Outlook/XML e ligação a servidores de bases de dados.

## Compilação (FreeBSD / X11)

Requisitos: FreeBSD com ambiente gráfico X11, Rust (edition 2021) e
Cargo.

``` sh
pkg install rust
git clone https://github.com/Olen1962/accessql.git
cd accessql
cargo build --release
./target/release/accessql
```

Para compilar em modo de desenvolvimento e executar os testes:

``` sh
cargo build
cargo test
```

Dependências principais:

-   `eframe` / `egui` 0.29 --- interface gráfica;
-   `rusqlite` 0.32 com a funcionalidade `bundled` --- acesso ao SQLite;
-   `rfd` 0.15 --- diálogos de ficheiros.

O diretório `target/` não é versionado.

## Utilização

1.  Em **Ficheiro → Novo**, escolher o nome e a pasta da nova base de
    dados `.accsql`. A opção **Abrir** aceita ficheiros SQLite
    existentes e recusa ficheiros que não sejam bases de dados válidas.
2.  No painel de navegação, fazer duplo clique numa tabela ou consulta
    para a abrir. O menu de contexto permite aceder às vistas
    disponíveis, mudar o nome, duplicar ou eliminar o objeto.
3.  Na Folha de Dados, editar os valores diretamente. As alterações são
    gravadas imediatamente.
4.  Para criar ou modificar tabelas, utilizar a Vista Estrutura. Para
    executar instruções SQL, abrir uma Consulta SQL.
5.  Para criar uma cópia de segurança, utilizar **Guardar Cópia Como**.

### Atalhos principais

  -----------------------------------------------------------------------
  Atalho                              Ação
  ----------------------------------- -----------------------------------
  `F2`                                Editar o nome do objeto, o campo ou
                                      a célula, conforme o foco

  `Ctrl+D`                            Duplicar o objeto selecionado no
                                      painel de navegação

  `F5` ou `Ctrl+Enter`                Executar SQL

  `Ctrl+C` / `Ctrl+X` / `Ctrl+V`      Copiar / cortar / colar

  `Ctrl+'`                            Copiar o valor da célula
                                      imediatamente acima

  `Ctrl+Z` / `Ctrl+Y`                 Desfazer / refazer

  `Delete`                            Limpar a célula ou eliminar o
                                      registo/objeto selecionado,
                                      conforme o foco

  `Ctrl` + roda do rato               Ajustar o zoom da interface
  -----------------------------------------------------------------------

## Licença

Distribuído sob os termos da **GNU General Public License v3.0 ou
posterior** --- consultar [LICENSE](LICENSE).

As dependências têm as suas próprias licenças: egui/eframe (MIT ou
Apache-2.0), rusqlite (MIT), rfd (MIT ou Apache-2.0) e SQLite (domínio
público). Consultar os ficheiros de licença das respetivas
distribuições.
