# Prompts para extrair spec de reimplementação (stack-agnóstico)

Objetivo: rodar estes prompts num agente que tem acesso ao código-fonte do app original
e obter um blueprint completo — funcional + técnico — que permita reescrever o app do zero
em **qualquer stack**, sem o agente assumir nada do stack atual como obrigatório.

Como usar:
1. Abra o repositório original no agente (Claude Code / Cowork) com acesso de leitura aos arquivos.
2. Rode os prompts **na ordem**. Cada um gera um artefato `.md` que o próximo consome.
3. No fim você terá uma pasta `spec/` com tudo. Leve essa pasta pro projeto novo.

> Dica: se o app for pequeno (poucos milhares de linhas), você pode usar só o **Prompt 0 (master)**.
> Se for médio/grande, use os prompts 1→4 encadeados — o resultado é mais completo e menos alucinado.

---

## Prompt 0 — Master (single-shot, para apps pequenos)

```
Você é um arquiteto de software fazendo engenharia reversa deste repositório para
produzir uma SPEC DE REIMPLEMENTAÇÃO agnóstica de stack. Eu vou reescrever este app
do zero em outra tecnologia (ainda não decidida), então NÃO trate o stack atual como
requisito — descreva o COMPORTAMENTO e as REGRAS, separando claramente o "o quê/por quê"
(essencial, portável) do "como" (detalhe de implementação do stack atual, descartável).

Antes de escrever, faça um reconhecimento real do código:
- Liste linguagens, frameworks, libs principais e gerenciadores de pacote (leia os
  manifests: package.json, pyproject.toml, go.mod, pom.xml, Gemfile, etc.).
- Identifique os entrypoints (main, server, rotas, CLI, cron) e como o app sobe e roda.
- Mapeie a estrutura de pastas em 1 parágrafo.

Depois produza um único documento markdown com estas seções:

1. VISÃO GERAL — o que o app faz, para quem, e o problema que resolve (3-6 frases).
2. INVENTÁRIO DE FUNCIONALIDADES — lista de features. Para cada uma: nome, descrição,
   gatilho (como o usuário/sistema inicia), e resultado esperado.
3. FLUXOS DE USUÁRIO — passo a passo dos principais fluxos, incluindo telas/endpoints
   envolvidos e ramificações (sucesso, erro, casos de borda).
4. REGRAS DE NEGÓCIO — toda lógica não óbvia: validações, cálculos, condições,
   defaults, limites. Cite o arquivo/função de onde tirou cada regra.
5. MODELO DE DADOS — entidades, campos (com tipo e significado), relacionamentos,
   chaves, índices relevantes. Inclua um diagrama em Mermaid (erDiagram).
6. CONTRATOS / APIs — endpoints ou interfaces públicas: método, rota, request, response,
   códigos de erro. Se for app só de UI, descreva as ações e seus contratos internos.
7. INTEGRAÇÕES EXTERNAS — APIs de terceiros, bancos, filas, storage, e-mail, etc.:
   qual, para quê, como autentica, e o que acontece se falhar.
8. CONFIGURAÇÃO & AMBIENTE — variáveis de ambiente, secrets (só os NOMES, nunca valores),
   feature flags, e arquivos de config. O que cada um controla.
9. JOBS / TAREFAS EM BACKGROUND — schedulers, workers, crons: o que rodam e quando.
10. ALGORITMOS / LÓGICA CENTRAL — qualquer parte "core" que dá identidade ao app
    (parsing, scoring, matching, geração, etc.) descrita em pseudocódigo agnóstico.
11. EXPERIÊNCIA (UX/CX/VISUAL) — identidade visual (design tokens: cores com valores,
    tipografia, espaçamento), componentes de UI, microinterações/animações (gatilho,
    duração, easing), estados de UI (vazio/loading/erro/sucesso), copy e tom de voz,
    acessibilidade/responsividade/temas, e assets (ícones, sons, animações). E procure
    ATIVAMENTE por EASTER EGGS e detalhes escondidos: sequências de teclado, cliques
    repetidos, comandos secretos, datas especiais, mensagens no console, conteúdo
    condicional raro. Grep por "easter", "secret", "konami", "confetti", listeners de
    keydown incomuns e condicionais com constantes mágicas. Não descarte como "código
    morto" sem investigar. Cite arquivo:linha.
12. REQUISITOS NÃO-FUNCIONAIS observáveis no código — performance, concorrência,
    autenticação/autorização, tratamento de erro, logging, persistência local.
13. CRITÉRIOS DE ACEITAÇÃO — para cada feature do item 2, escreva 1-3 asserções
    verificáveis no formato "Dado/Quando/Então", para validar a reimplementação.
14. PREMISSAS & LACUNAS — tudo que você inferiu mas não conseguiu confirmar no código,
    e perguntas que eu (o dono) preciso responder.

Regras de saída:
- Seja concreto e cite caminhos de arquivo (caminho:linha quando possível).
- Não invente. Se não encontrou, escreva "NÃO ENCONTRADO NO CÓDIGO".
- Nunca inclua valores de secrets, tokens ou dados pessoais — apenas os nomes/papéis.
- Markdown limpo, sem floreio. Salve como spec/SPEC-COMPLETA.md.
```

---

## Prompt 1 — Reconhecimento técnico (base factual)

```
Faça um reconhecimento técnico SOMENTE FACTUAL deste repositório. Não opine, não
projete reimplementação ainda — só descreva o que existe, citando arquivos.

Entregue em markdown (salve como spec/01-reconhecimento.md):

A) STACK: linguagens, frameworks, libs relevantes e versões (leia os manifests).
B) ESTRUTURA: árvore de pastas resumida + papel de cada diretório de topo.
C) COMO RODA: comandos de build/run/test, entrypoints, portas, processos.
D) PERSISTÊNCIA: bancos, ORMs, migrações, arquivos locais, caches.
E) SUPERFÍCIE EXTERNA: rotas/endpoints/CLIs expostos (apenas listar, com arquivo de origem).
F) INTEGRAÇÕES: toda chamada a serviço externo encontrada (qual lib, qual arquivo).
G) CONFIG: nomes de env vars e flags lidos pelo código (NUNCA valores).

Para cada item, cite o(s) arquivo(s). Onde não houver, escreva "NÃO ENCONTRADO".
```

---

## Prompt 2 — Extração funcional (o quê e por quê)

```
Usando o repositório E o documento spec/01-reconhecimento.md como contexto, extraia a
camada FUNCIONAL do app, de forma agnóstica de tecnologia. Pense como quem vai reescrever:
descreva comportamento e regras, não implementação.

Entregue em markdown (salve como spec/02-funcional.md):

1. VISÃO GERAL — propósito, usuário-alvo, problema resolvido.
2. FEATURES — tabela: nome | descrição | gatilho | resultado | arquivos de origem.
3. FLUXOS DE USUÁRIO — passo a passo dos fluxos principais, com ramificações de erro
   e casos de borda. Indique telas/endpoints envolvidos.
4. REGRAS DE NEGÓCIO — toda lógica não óbvia (validações, cálculos, defaults, limites),
   cada uma com a citação do arquivo/função de origem.
5. ESTADOS & CICLOS DE VIDA — máquinas de estado relevantes (ex: status de um pedido).
6. PERMISSÕES — quem pode fazer o quê, se houver papéis/autorização.

Não descreva detalhes de framework. Se uma regra estiver acoplada ao stack, extraia a
INTENÇÃO. Marque inferências com "[INFERIDO]" e o que não achou com "NÃO ENCONTRADO".
```

---

## Prompt 2.5 — Extração de UX / CX / visual / easter eggs

```
Usando o repositório + spec/02-funcional.md, extraia a camada de EXPERIÊNCIA do app:
visual, UX, CX e tudo que dá personalidade. Esta parte é fácil de perder porque vive
espalhada em CSS, assets, strings e condicionais escondidas — vasculhe a fundo.

Entregue em markdown (salve como spec/025-experiencia.md):

1. IDENTIDADE VISUAL — design tokens: paleta de cores (com os valores hex/rgb),
   tipografia (famílias, pesos, escalas), espaçamento, raios de borda, sombras.
   Leia CSS/SCSS/Tailwind config/theme files e liste os tokens reais, citando arquivo.
2. LAYOUT & COMPONENTES — estrutura de telas, grid, componentes de UI reutilizáveis
   e suas variações (botões, cards, modais, navegação). Descreva hierarquia visual.
3. INTERAÇÕES & MICROINTERAÇÕES — hovers, transições, animações, loading states,
   feedback de toque/clique, drag-and-drop, gestos. Para cada animação: gatilho,
   duração, easing, e o que ela comunica. Cite o arquivo.
4. ESTADOS DE UI — vazio (empty state), carregando, erro, sucesso, offline. Como cada
   um é apresentado ao usuário (texto + visual).
5. COPY & TOM DE VOZ — estilo das mensagens (formal/informal/bem-humorado), padrões de
   microcopy (botões, placeholders, toasts, erros). Extraia exemplos reais de strings.
6. EASTER EGGS & DETALHES ESCONDIDOS — procure ATIVAMENTE por: sequências de teclado
   (ex: Konami code), cliques repetidos, comandos secretos, datas especiais, mensagens
   no console, gatilhos por valores específicos, animações raras, conteúdo condicional.
   Grep por termos como "easter", "secret", "konami", "🎉", "confetti", "surprise",
   listeners de keydown incomuns, e condicionais com constantes mágicas. Para cada um:
   como dispara, o que acontece, e o arquivo:linha.
7. ACESSIBILIDADE & RESPONSIVIDADE — breakpoints, comportamento mobile/desktop, suporte
   a teclado, ARIA, dark mode / temas. Apenas o que se observa no código.
8. ASSETS — ícones, ilustrações, fontes, sons, lottie/json de animação: liste os
   arquivos e onde são usados (eu vou precisar reexportar/recriar no novo stack).

Cite arquivo:linha sempre. Para easter eggs, NÃO descarte nada como "código morto"
sem investigar — o ponto é justamente capturar o que parece escondido de propósito.
Marque "[INFERIDO]" / "NÃO ENCONTRADO".
```

---

## Prompt 3 — Extração técnica (detalhe portável)

```
Usando o repositório + spec/01-reconhecimento.md + spec/02-funcional.md, extraia a
camada TÉCNICA necessária para reimplementar, mantendo-a portável (sem depender de um
framework específico).

Entregue em markdown (salve como spec/03-tecnico.md):

1. MODELO DE DADOS — entidades, campos (tipo + significado), relações, chaves, índices.
   Inclua diagrama Mermaid erDiagram.
2. CONTRATOS / APIs — para cada endpoint/interface: método, rota, parâmetros, corpo de
   request, corpo de response, erros. Em pseudo-schema neutro (não OpenAPI de um framework).
3. INTEGRAÇÕES — para cada serviço externo: propósito, payloads de entrada/saída,
   autenticação (mecanismo, não credencial), comportamento em falha/retry/timeout.
4. ALGORITMOS CENTRAIS — a lógica que dá identidade ao app, em pseudocódigo agnóstico,
   passo a passo, com complexidade quando relevante. Cite a origem.
5. JOBS/BACKGROUND — o que roda fora do request/response e em qual cadência.
6. REQUISITOS NÃO-FUNCIONAIS — auth/authz, concorrência, performance, idempotência,
   logging, tratamento de erro, observabilidade — apenas o que se observa no código.
7. CONFIGURAÇÃO — tabela: nome da env/flag | o que controla | obrigatória? | default.
   Apenas nomes e papéis, nunca valores ou secrets.

Cite arquivo:linha sempre que possível. Marque "[INFERIDO]" / "NÃO ENCONTRADO".
```

---

## Prompt 4 — Montagem da spec final + critérios de aceitação

```
Consolide spec/01-reconhecimento.md, spec/02-funcional.md, spec/025-experiencia.md e
spec/03-tecnico.md em uma única SPEC DE REIMPLEMENTAÇÃO autossuficiente, pensada para alguém reescrever o app do
zero em um stack ainda não definido. Não copie e cole cru — integre, remova redundância
e resolva contradições entre os documentos (se houver, aponte qual venceu e por quê).

Entregue em markdown (salve como spec/SPEC-REIMPLEMENTACAO.md) com:

- Sumário executivo (o que é, o que faz, escopo).
- Todas as seções funcionais e técnicas integradas e organizadas.
- CRITÉRIOS DE ACEITAÇÃO: para CADA feature, asserções verificáveis no formato
  "Dado / Quando / Então" que provem que a reimplementação está correta.
- MATRIZ DE PRIORIDADE: classifique cada feature como Núcleo (MVP) / Importante /
  Acessório, para guiar a ordem de reconstrução.
- DECISÕES A TOMAR NO NOVO STACK: pontos onde o stack atual fez uma escolha que o novo
  precisará refazer (ex: ORM, fila, auth), apresentados como decisão aberta, não receita.
- PREMISSAS & PERGUNTAS ABERTAS: tudo que ficou inferido ou ambíguo, em forma de
  checklist para eu responder antes de começar.

Critérios de qualidade: nada de detalhe de implementação amarrado ao stack antigo na
parte de requisitos; toda regra rastreável à origem; nenhum secret/valor sensível.
```

---

## Prompt 5 (opcional) — Auditoria da spec antes de codar

```
Aja como revisor cético. Compare spec/SPEC-REIMPLEMENTACAO.md com o repositório real e
encontre LACUNAS e ERROS antes de eu começar a reescrever. Procure por:
- Features ou rotas que existem no código mas faltam na spec.
- Regras de negócio descritas errado ou simplificadas demais.
- Casos de borda e tratamento de erro que a spec ignorou.
- Dependências escondidas (ordem de inicialização, estado global, efeitos colaterais).
- Qualquer "magia" do framework atual que a spec assumiu como dada e que o novo stack
  terá que implementar explicitamente.

Entregue uma lista priorizada (Crítico/Médio/Baixo) de correções a fazer na spec,
cada item com a evidência no código (arquivo:linha). Salve como spec/04-auditoria.md.
```

---

## Notas de uso

- **Secrets/dados sensíveis:** todos os prompts já instruem o agente a registrar só os
  *nomes* de variáveis e secrets, nunca valores. Mesmo sendo seu app pessoal, é bom hábito.
- **Rastreabilidade:** a exigência de citar `arquivo:linha` é o que evita alucinação —
  se o agente não cita a origem, desconfie da regra.
- **Stack de destino:** quando você decidir o stack novo, vale um prompt extra pedindo o
  *mapeamento origem→destino* (ex: "Postgres+SQLAlchemy" → "qual equivalente no novo stack")
  e um plano de migração incremental. Posso te montar esse quando você escolher.
```
