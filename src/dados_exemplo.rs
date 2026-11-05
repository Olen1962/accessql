use crate::tema::*;

// ───────────────────────────── Dados de exemplo (em memória) ─────────────────────────────

pub(crate) type Cols = Vec<(&'static str, f32)>;

pub(crate) type Rows = Vec<Vec<&'static str>>;



pub(crate) fn dataset(name: &str) -> (Cols, Rows) {

    match name {

        "Produtos" | "Produtos em Falta" => (

            vec![("ID", 60.0), ("Produto", 210.0), ("Categoria", 130.0), ("Preço", 90.0), ("Stock", 80.0)],

            vec![

                vec!["1", "Caneta esferográfica", "Papelaria", "0,85 €", "420"],

                vec!["2", "Caderno A4 pautado", "Papelaria", "2,40 €", "185"],

                vec!["3", "Monitor 24\"", "Informática", "139,00 €", "12"],

                vec!["4", "Teclado mecânico", "Informática", "59,90 €", "27"],

                vec!["5", "Cadeira de escritório", "Mobiliário", "119,00 €", "9"],

                vec!["6", "Candeeiro LED", "Mobiliário", "24,50 €", "64"],

                vec!["7", "Pen USB 64 GB", "Informática", "9,99 €", "230"],

            ],

        ),

        "Encomendas" | "Encomendas por Cliente" => (

            vec![("ID", 60.0), ("Cliente", 160.0), ("Data", 100.0), ("Total", 100.0), ("Estado", 110.0)],

            vec![

                vec!["1001", "Ferreira & Filhos Lda", "2026-10-02", "312,40 €", "Enviada"],

                vec!["1002", "Carvalho Têxteis", "2026-10-05", "1 280,00 €", "Paga"],

                vec!["1003", "Mendes Consultores", "2026-10-11", "89,90 €", "Pendente"],

                vec!["1004", "Santos Logística", "2026-10-19", "540,25 €", "Enviada"],

                vec!["1005", "Costa Vinhos", "2026-10-27", "2 015,00 €", "Pendente"],

                vec!["1006", "Lopes Editores", "2026-11-01", "76,00 €", "Paga"],

            ],

        ),

        _ => (

            vec![("ID", 60.0), ("Nome", 110.0), ("Apelido", 120.0), ("Empresa", 170.0), ("Cidade", 110.0), ("País", 90.0), ("Telefone", 120.0)],

            vec![

                vec!["1", "Ana", "Ferreira", "Ferreira & Filhos Lda", "Lisboa", "Portugal", "210 000 001"],

                vec!["2", "Bruno", "Carvalho", "Carvalho Têxteis", "Porto", "Portugal", "220 000 002"],

                vec!["3", "Carla", "Mendes", "Mendes Consultores", "Coimbra", "Portugal", "239 000 003"],

                vec!["4", "Diogo", "Santos", "Santos Logística", "Braga", "Portugal", "253 000 004"],

                vec!["5", "Eva", "Pereira", "Pereira Engenharia", "Faro", "Portugal", "289 000 005"],

                vec!["6", "Fábio", "Costa", "Costa Vinhos", "Évora", "Portugal", "266 000 006"],

                vec!["7", "Graça", "Lopes", "Lopes Editores", "Setúbal", "Portugal", "265 000 007"],

                vec!["8", "Hugo", "Martins", "Martins Informática", "Aveiro", "Portugal", "234 000 008"],

            ],

        ),

    }

}



pub(crate) fn dd_items(key: &str) -> &'static [&'static str] {

    match key {

        "vista" => &["Vista Folha de Dados", "Vista de Esquema", "Vista Estrutura"],

        "colar" => &["Colar", "Colar Especial...", "Colar Acrescentar"],

        "sel" => &["Igual a \"valor\"", "Diferente de \"valor\"", "Contém \"valor\"", "Não Contém \"valor\""],

        "avanc" => &["Filtrar por Formulário", "Aplicar Filtro/Ordenação", "Limpar Todos os Filtros", "Filtro Avançado/Ordenação..."],

        "mais_reg" => &["Altura da Linha...", "Largura da Coluna...", "Ocultar Campos", "Mostrar Campos..."],

        "goto" => &["Primeiro", "Último", "Seguinte", "Anterior", "Novo"],

        "selecionar" => &["Selecionar", "Selecionar Tudo"],

        "navegacao" => &["Separadores Horizontais", "Separadores Verticais, Esquerda", "Separadores Verticais, Direita"],

        "mais_forms" => &["Vários Itens", "Folha de Dados", "Formulário Dividido", "Caixa de Diálogo Modal"],

        "modelos" => &["Contactos", "Tarefas", "Eventos", "Problemas"],

        "partes" => &["Comentários", "Contactos", "Utilizadores", "Início Rápido"],

        "imp_mais" => &["Ficheiro XML", "Documento HTML", "Pasta do Outlook", "Ficheiro dBASE"],

        "exp_mais" => &["Word", "Ficheiro HTML", "Ficheiro dBASE", "Pasta do Outlook"],

        "tipo" => &["Texto Curto", "Texto Longo", "Número", "Data/Hora", "Moeda", "Numeração Automática", "Sim/Não"],

        "navcat" => &["Tipo de Objeto", "Tabelas e Vistas Relacionadas", "Data de Criação", "Data de Modificação", "Filtrar por Grupo"],

        _ => &["(apenas apresentação visual)"],

    }

}
