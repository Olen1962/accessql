# AccesSQL

Aplicação desktop para gerir bases de dados **SQLite locais**, com uma interface inspirada no
**Microsoft Access 2007–2010** (ribbon, painel de navegação, folha de dados e backstage «Ficheiro»).

Tudo é local e offline: não há contas, rede nem pesquisas online. Cada base de dados é um ficheiro
SQLite normal com extensão `.accsql`, pelo que pode ser aberta por qualquer ferramenta SQLite.

> **Não afiliação:** o AccesSQL não está afiliado, endossado nem patrocinado pela Microsoft
> Corporation. «Microsoft Access» é uma marca registada da Microsoft Corporation. Este projeto
> não contém código, componentes ou dados do Microsoft Access — a semelhança visual existe apenas
> para dar familiaridade a quem usa esse tipo de ferramenta.

## Funcionalidades

- **Ficheiro:** criar, abrir, recentes, informações da base, compactar/reparar (`integrity_check` +
  `VACUUM`), «Guardar Cópia Como» e fechar. A gravação é automática — não existe «Guardar».
- **Tabelas e consultas:** o painel de navegação lê o catálogo SQLite; as consultas guardadas são
  `VIEW`s. Cada objeto tem um separador, que alterna entre Folha de Dados, Estrutura e SQL.
- **Folha de dados:** editar células, linha `(Novo)`, eliminar registos, ordenar, filtrar, procurar
  com realce, totais da coluna e validação de tipos ao gravar (também na colagem e no CSV).
- **Estrutura:** criar tabelas, acrescentar/renomear/eliminar campos, chaves primárias, valores por
  omissão e chaves estrangeiras (coluna «Refere a»).
- **SQL:** editor com `F5`/`Ctrl+Enter`, scripts com várias instruções e resultados em grelha.
- **Dados externos:** importar e exportar CSV, com deteção de separador e codificação.
- **Ferramentas:** relações (chaves estrangeiras), índices e eliminação segura de objetos.
- **Edição:** copiar/cortar/colar (células, registos inteiros e múltiplas linhas), `Ctrl+'`, F2
  universal e Desfazer/Refazer.

Fora do âmbito desta versão: formulários, relatórios, macros, importação de Excel/Outlook/XML e
servidores de bases de dados.

## Compilação (FreeBSD / X11)

Necessário: Rust (edition 2021), `cargo` e X11.

```sh
pkg install rust cargo
git clone https://github.com/Olen1962/accessql.git
cd accessql
cargo build --release
./target/release/accessql
```

Para compilar em modo de desenvolvimento e correr os testes:

```sh
cargo build
cargo test
```

As dependências principais são `eframe`/`egui` 0.29 (interface), `rusqlite` 0.32 com SQLite
«bundled» (dados) e `rfd` 0.15 (diálogos de ficheiros). O `target/` não é versionado.

## Utilização

1. **Ficheiro → Novo** escolhe o nome e a pasta da nova base `.accsql`; **Abrir** aceita ficheiros
   SQLite existentes (recusa ficheiros que não sejam bases de dados).
2. No painel de navegação, duplo clique abre uma tabela ou consulta; o botão direito permite abrir
   a estrutura, o SQL, mudar o nome (`F2`) ou eliminar.
3. A gravação é imediata. Para uma cópia de segurança, usar **Guardar Cópia Como**.

Atalhos úteis: `F2` (editar nome/célula/campo), `F5` ou `Ctrl+Enter` (executar SQL), `Ctrl+C` /
`Ctrl+X` / `Ctrl+V` (área de transferência), `Ctrl+'` (copiar valor da célula acima), `Ctrl+Z` /
`Ctrl+Y` (desfazer/refazer), `Delete` (limpar célula ou eliminar registo/objeto selecionado).

## Licença

Distribuído sob os termos da **GNU General Public License v3.0 ou posterior** — ver [LICENSE](LICENSE).

As dependências têm as suas próprias licenças: egui/eframe (MIT ou Apache-2.0), rusqlite (MIT),
rfd (MIT ou Apache-2.0) e SQLite (domínio público). Consulte os ficheiros de licença incluídos
nas respetivas distribuições via `cargo`.
