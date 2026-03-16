# YGN — Brief opératoire v2 pour Claude Code 4.6 (1M contexte)

_Date de référence : 15 mars 2026_

## 0. Rôle

Tu agis comme **Chief Product + Staff Engineer + GTM strategist**.

Ta mission n’est **pas** de proposer une jolie vision générale.
Ta mission est de **convertir un portefeuille de repos techniques en un business durable, différencié et rentable**, avec une exécution réaliste pour un fondateur solo disposant de peu de budget et d’un temps limité.

Tu dois raisonner avec **brutalité stratégique** :
- couper les idées séduisantes mais non défendables,
- éviter les paris frontaux contre les gros acteurs,
- privilégier la wedge la plus vendable avec les actifs existants,
- préserver la marge,
- réduire le support,
- limiter les coûts d’inférence,
- maximiser la vitesse de mise sur le marché.

## 1. Ce qui change par rapport au brief précédent

Le brief précédent était directionnellement bon, mais trop large. Il poussait encore vers un **"control plane de gouvernance d’agents"** assez général.

Après audit technique des repos et revue de marché, la conclusion corrigée est la suivante :

### Verdict

1. **La catégorie "agent control plane / governance" n’est plus vide.**
   Elle est désormais occupée par des acteurs crédibles et financés, ainsi que par des plateformes intégrées.

2. **Le bon wedge n’est pas la gouvernance générique.**
   Le bon wedge est :

> **Verifiable proof for coding-agent compliance**
>
> en français : **preuve vérifiable des actions d’agents de code**.

3. **Il ne faut pas vendre "des guardrails" comme produit principal.**
   Les guardrails, policies et patterns de blocage sont utiles, mais déjà largement commoditisés.

4. **La vraie valeur défendable est la preuve.**
   Plus précisément :
   - reçus / artefacts vérifiables,
   - intégrité cryptographique,
   - hash chains,
   - signatures,
   - evidence packs,
   - cross-validation inter-modèles,
   - export d’artefacts auditables.

5. **Le produit principal ne doit pas être un plugin Claude Code payant à bas prix.**
   Le plugin peut devenir un excellent **lead magnet**, mais pas le cœur du revenu long terme.

## 2. Réalité marché à intégrer comme contrainte dure

Tu dois partir de ces faits comme d’un environnement imposé.

### 2.1. La concurrence sur la gouvernance d’agents est réelle

- **GitHub** a mis en disponibilité générale ses **Enterprise AI Controls & agent control plane** fin février 2026, avec gestion centralisée des agents, politiques IA, sessions d’agents et audit logs.
- **Microsoft Agent 365** sera disponible le **1er mai 2026**, avec sécurité, least-privilege, protection des données, gestion d’agents et gouvernance.
- **Runlayer** s’est positionné comme couche de sécurité et d’infrastructure MCP/agents, a annoncé **11 M$** levés, met en avant **threat detection**, **fine-grained permissions**, **observability**, **SOC 2** et **HIPAA**, ainsi que des clients déjà en production.
- **Singulr AI** vient de lancer **Agent Pulse** début mars 2026, avec runtime governance, discovery, risk intelligence, policy enforcement et runtime controls pour agents et MCP.
- **MintMCP** se positionne aussi fortement sur la gouvernance/monitoring/gateway MCP, avec **SOC 2 Type II** en avant sur ses supports publics.

### 2.2. Conséquence stratégique

Ne construis **pas** un pitch du type :
- “nous faisons un control plane pour agents”,
- “nous faisons de la gouvernance MCP”,
- “nous faisons des guardrails pour Claude/Cursor/Copilot”.

Ce positionnement te mettrait en concurrence trop frontale avec :
- plateformes intégrées (GitHub, Microsoft),
- vendors financés (Runlayer, Singulr, MintMCP),
- stack enterprise vendue sur sécurité/compliance/identity.

### 2.3. Nuance importante

Les concurrents publics mettent surtout en avant :
- visibilité,
- contrôle,
- permissions,
- auditabilité,
- observabilité,
- policy enforcement,
- intégration identité.

Ils mettent **beaucoup moins** en avant, dans leurs messages publics, une couche de **preuve cryptographique vérifiable** des actions d’agents.

Cela ne veut **pas** dire que personne n’y pense.
Cela veut dire :

> **la preuve vérifiable est un wedge plus étroit, plus distinctif, et moins frontalement occupé que la gouvernance générique.**

## 3. Ce qui a été confirmé dans les repos

Tu dois considérer les éléments ci-dessous comme confirmés et t’appuyer dessus.

### 3.1. KodoClaw

KodoClaw contient des briques utiles mais **pas suffisantes comme moat autonome**.

Éléments confirmés :
- scanner d’injection type **Aho-Corasick** avec normalisation d’homoglyphes Unicode et retrait de caractères zero-width,
- classification de risques shell par patterns,
- matrice de décision allow/confirm/block,
- output guard par regex pour patterns dangereux,
- vault chiffré en **XChaCha20-Poly1305**.

Conclusion produit :
- utile comme **surface d’intégration** avec Claude Code,
- utile comme **capteur** et point d’instrumentation,
- utile comme **lead magnet communautaire**,
- **pas** la base du pitch principal.

### 3.2. Meta-YGN / Aletheia-Nexus

Meta-YGN contient les briques qui justifient le pivot vers la preuve.

Éléments confirmés :
- guard pipeline Rust composable,
- evidence pack avec :
  - **SHA-256 hash chain** via `prev_hash`,
  - **Merkle root**,
  - **signatures Ed25519** via `ed25519_dalek`,
  - vérification d’intégrité de chaîne,
- session replay,
- vérifications d’intégrité / completion.

Conclusion produit :
- c’est **le noyau différenciant**,
- c’est la base d’un **flight recorder / proof layer** pour agents de code,
- c’est le repo le plus important à transformer en produit premium.

### 3.3. nexus-evidence

Éléments confirmés :
- dual-agent review **Claude + Gemini**,
- evidence packs **SHA-256 verified**,
- PR review orientée artefacts.

Conclusion produit :
- très bon angle pour la **preuve inter-modèles**,
- très bon composant pour un **mode premium / compliance review**,
- bon packaging possible en CLI, sidecar CI ou service.

### 3.4. YGN-SAGE

Éléments confirmés :
- repo volumineux,
- architecture riche,
- benchs, routing cognitif, guardrails, sandbox, dashboard,
- README explicite : **research prototype, not production-ready**.

Conclusion produit :
- **ne pas vendre le monolithe**,
- l’utiliser comme **réservoir de composants et d’idées**,
- ne sortir que des morceaux ciblés si cela sert le wedge principal.

### 3.5. Signal traction

Les repos clés visibles publiquement ont encore une traction ouverte faible (ex. 0 star / 0 fork sur plusieurs repos majeurs).

Interprétation imposée :
- le problème n’est pas l’absence de profondeur technique,
- le problème est le **packaging**, la **distribution**, le **positionnement**, et la **preuve de valeur achetable**.

## 4. Positionnement final à adopter

### 4.1. Proposition de valeur

Le produit ne doit pas être présenté comme :
- un framework multi-agent,
- de la métacognition pour agents,
- un plugin Claude Code plus malin,
- une alternative à Runlayer / GitHub / Microsoft.

Le produit doit être présenté comme :

> **La couche de preuve vérifiable pour agents de code**
>
> _Cryptographic evidence, signed receipts, and tamper-evident audit artifacts for coding-agent actions._

### 4.2. Phrase de vente simple

Proposer plusieurs formulations et choisir la meilleure après tests.

Version FR :
- **Prouvez ce que vos agents de code ont réellement fait.**
- **Des logs ne suffisent pas. Générez des preuves vérifiables.**
- **Receipts signés, chaîne d’intégrité, evidence packs pour Claude Code, Copilot et Cursor.**

Version EN :
- **Don’t just log agent actions. Prove them.**
- **Cryptographic evidence for coding-agent compliance.**
- **Signed receipts and tamper-evident evidence packs for AI coding workflows.**

### 4.3. ICP (Ideal Customer Profile)

Ne vise pas d’abord les très grandes entreprises US qui achèteront Runlayer/Microsoft/GitHub.

ICP prioritaire :
- **PME / scale-ups françaises et européennes**,
- équipes de **20 à 300 développeurs**,
- déjà en pilote ou en déploiement de Claude Code / Copilot / Cursor / MCP,
- sensibles à la conformité, aux audits, aux revues sécurité, à la traçabilité,
- pas assez grosses pour signer facilement des deals lourds avec les gros vendors,
- ou voulant une solution plus légère / plus locale / self-hosted.

Sous-segments prioritaires :
1. B2B SaaS soumis à exigences SOC 2 / ISO 27001 / revues sécurité client.
2. Healthtech / fintech logicielle / legaltech / govtech / industrie logicielle.
3. Équipes françaises/européennes ayant une exigence forte de **preuve**, pas seulement de monitoring.
4. En option stratégique : verticale **industrial / MES / ISA-88/95 / OT software**, où le fondateur a une crédibilité rare.

### 4.4. Pourquoi ce wedge peut se vendre

Parce qu’il répond à un besoin concret :
- démontrer à un audit interne ce qui s’est passé,
- démontrer à un client enterprise qu’un workflow agentique est encadré,
- montrer qu’une action d’agent est traçable, non altérée et attribuable,
- fournir des artefacts exportables plutôt que des logs bruts illisibles,
- différencier une simple observabilité d’une **preuve d’intégrité**.

## 5. Produit à construire

## Produit principal : **Aletheia Proof** (nom de travail)

### 5.1. Nature du produit

Un **proof layer / sidecar / flight recorder** pour coding agents.

Le produit s’insère entre :
- l’agent (Claude Code, Copilot, Cursor, Codex CLI, Agent SDK, etc.),
- les outils / MCP servers / shell / workflows CI,
- et la couche de sortie (PR, rapport, export, audit).

### 5.2. Ce que le produit doit faire en v1

Fonctions minimum :

1. **Capturer** les actions d’agent pertinentes.
2. **Créer des receipts signés** pour les événements critiques.
3. **Chaîner** les événements avec hash chain.
4. **Produire un evidence pack exportable**.
5. **Vérifier l’intégrité** d’un pack après coup.
6. **Associer l’action à un contexte** : session, repo, PR, outil, policy, résultat.
7. **Rendre les artefacts lisibles** pour un humain non expert.

### 5.3. Livrables techniques v1

Le MVP produit doit sortir sous 3 formes compatibles :

#### A. CLI local
- génération d’evidence pack pour une session ou une PR,
- commande de vérification,
- export JSON + HTML/Markdown lisible.

#### B. Sidecar / proxy léger
- placé entre l’agent et certains outils/MCP/tool calls,
- journalise et scelle les événements clés.

#### C. Intégration CI / PR
- exécute review + génération de preuve sur PR,
- publie un artifact téléchargeable,
- éventuellement commentaire PR avec digest et lien.

### 5.4. V2 possible

Plus tard seulement :
- dashboard minimal,
- policy manifests signés,
- mode cross-validation multi-modèle,
- exports orientés audit/compliance,
- self-hosted team server,
- connectors Slack/Jira/GitHub/GitLab.

### 5.5. Ce que le produit n’est PAS

- pas un agent autonome généraliste,
- pas un framework multi-agent,
- pas une plateforme de gouvernance complète,
- pas un SOC 2 vendor out-of-the-box,
- pas un concurrent direct de Runlayer ou Agent 365,
- pas une marketplace de plugins.

## 6. Offre commerciale à lancer

## 6.1. Modèle commercial recommandé

Commencer par **service productisé + produit self-hosted léger**.

Pas par un SaaS large.
Pas par un abonnement plugin low-ticket.

### Offre 1 — Agent Evidence Sprint

But : cash rapide + apprentissage client + cas réels.

Contenu :
- audit d’un workflow agentique ou d’un repo,
- instrumentation de la preuve sur 1 à 2 flows réels,
- génération d’evidence packs,
- export demo + documentation courte,
- recommandations de policy et d’intégration.

Prix cible initial :
- **2 500 € à 5 000 €** selon périmètre.

### Offre 2 — Aletheia Proof Self-Hosted

Contenu :
- CLI/sidecar/installable,
- verification tooling,
- evidence pack export,
- support d’installation,
- mise à jour et assistance limitée.

Prix cible initial :
- **500 € à 1 500 € / mois** selon équipe,
- ou **6 000 € à 18 000 € / an** pour formule annuelle.

### Offre 3 — Compliance / Review Add-on

Contenu :
- dual-agent cross-validation,
- evidence pack premium,
- export orienté audit ou revue de sécurité,
- accompagnement de preuve pour appels d’offres / questionnaires sécurité / due diligence.

Prix cible :
- add-on projet ou abonnement premium.

## 6.2. Pourquoi pas KodoClaw Pro comme offre cœur

KodoClaw Pro peut exister, mais **seulement comme support d’acquisition**.

Raisons :
- pricing trop faible pour compenser le support,
- dépendance forte à l’écosystème plugin Claude Code,
- marché vite saturé,
- moat faible face aux évolutions natives des plateformes,
- meilleure utilité comme porte d’entrée vers le produit premium de preuve.

Décision :
- KodoClaw Community = lead magnet,
- KodoClaw Pro éventuel = option secondaire,
- **Aletheia Proof = cœur du business.**

## 7. Paiement et contrainte PayPal

Contrainte utilisateur : idéalement, les revenus doivent finir côté PayPal.

### Fait important

Avec Stripe + PayPal :
- Stripe permet **d’accepter PayPal**,
- Stripe permet aussi les **abonnements PayPal via Checkout**,
- pour les paiements PayPal, tu peux choisir la **settlement preference** :
  - vers le solde Stripe,
  - ou rester sur le solde PayPal.

### Conséquence opérationnelle

Si la contrainte dure est :
> “tout doit aller sur PayPal”

alors au lancement :
- **privilégier PayPal comme moyen de paiement principal dans Stripe Checkout**,
- et **settle on PayPal** pour les transactions PayPal.

Important :
- les paiements par carte n’atterriront pas naturellement sur PayPal via Stripe comme un flux homogène.
- Donc si l’exigence est littérale, il faut **soit** limiter les paiements au moyen PayPal au départ, **soit** accepter d’assouplir cette contrainte.

Décision recommandée :
- Lancement initial : **Checkout Stripe + PayPal activé + focus PayPal**, sans complexifier.

## 8. Repo mapping imposé

### 8.1. À mettre au cœur du produit

- **Meta-YGN** → noyau de preuve / receipts / intégrité / replay.
- **nexus-evidence** → evidence pack PR / dual-agent review / packaging export.
- **KodoClaw** → intégration Claude Code / hooks / capteurs / instrumentation.

### 8.2. À exploiter comme réserve interne

- **YGN-SAGE** → composants ciblés seulement.
- **metascaffold** → idées secondaires, pas front product.
- **Y-GN / NEXUS** → réserve R&D / contenu / documentation future.

### 8.3. À ne pas pousser commercialement maintenant

- la métacognition pure,
- les benchmarks auto-référentiels,
- le discours “S1/S2/S3” en façade commerciale,
- tout repo pouvant brouiller la vente enterprise/compliance.

## 9. Architecture v1 recommandée

### 9.1. Principe

Architecture la plus simple possible.

- site vitrine léger,
- Stripe Checkout,
- génération d’artefacts côté serveur ou local/self-hosted,
- pas de grosse plateforme multi-tenant au début.

### 9.2. Composants

- **Landing page** : simple, axée sur la preuve.
- **Checkout** : Stripe avec PayPal.
- **CLI / sidecar** : cœur du produit.
- **Artifact generation** : JSON + Markdown + HTML.
- **Verification command** : vérification d’intégrité.
- **PR integration** : GitHub Action / CI minimal.

### 9.3. Hébergement

Phase 1 :
- utiliser l’infra existante si suffisant.

Phase 2 :
- migrer sur GCP / Cloud Run si besoin.

### 9.4. Observabilité

Réutiliser OpenTelemetry si nécessaire, mais **ne pas faire de l’observabilité le cœur du pitch**.
La télémétrie sert le produit ; elle ne le définit pas.

## 10. Stratégie de distribution

## 10.1. Canal principal

Pas de vente via marketplace uniquement.

Canaux prioritaires :
- outreach ciblé LinkedIn / réseau / founders / CTO / Head of Eng / Platform,
- posts techniques ciblés sur :
  - “logs vs cryptographic proof for AI agents”,
  - “tamper-evident evidence packs for coding agents”,
  - “how to prove what Claude Code actually did”,
- démonstrations réelles sur PR / CI,
- cas d’usage européens / conformité / due diligence client.

## 10.2. Canal open source

L’open source sert de crédibilité et d’acquisition.

À publier / mettre en avant :
- un composant open-source étroit, utile et réutilisable,
- typiquement un **receipt verifier**, **proof pack format**, ou **CI action**.

Ce qu’il ne faut pas open-sourcer d’emblée :
- toute l’orchestration premium,
- tout ce qui dilue le différentiel commercial,
- les bundles de service, les templates premium, les workflows packagés.

## 10.3. Narratif distribution

Le narratif doit être simple :

> “Tout le monde parle de monitoring d’agents.
> Nous, on apporte la preuve vérifiable.”

## 11. Ce que tu dois produire maintenant

Tu dois travailler par étapes.

### Étape A — Décision de naming

Proposer 5 noms maximum, puis en retenir 1 principal et 1 backup.
Critères :
- crédibilité B2B,
- mémorisable,
- pas trop technique,
- compatible FR/EN,
- évoque preuve / intégrité / vérité / receipts.

### Étape B — Packaging produit

Produire :
1. tagline,
2. pitch 1 phrase,
3. pitch 30 secondes,
4. pitch landing page,
5. 3 use cases prioritaires,
6. 5 objections + réponses.

### Étape C — Spécification MVP

Définir précisément :
- événements capturés,
- structure des receipts,
- structure d’un evidence pack,
- commande de vérification,
- format de sortie lisible,
- intégration PR/CI,
- limites de v1.

### Étape D — Repo extraction plan

Faire un vrai mapping :
- fichier/source -> fonctionnalité cible,
- code réutilisable directement,
- code à refactorer,
- code à ignorer.

Tu dois éviter de dupliquer inutilement :
- réutiliser Meta-YGN pour le noyau de preuve,
- réutiliser nexus-evidence pour le packaging review,
- réutiliser KodoClaw pour l’instrumentation Claude Code.

### Étape E — Plan de commercialisation

Produire :
- ICP précis,
- pricing v1,
- structure de landing page,
- séquence d’outreach,
- offre pilote,
- critères de conversion.

## 12. Roadmap réaliste

## Semaines 1–2

Objectif : **preuve de produit**, pas scale.

Faire :
- choisir le nom,
- définir le format d’evidence pack,
- sortir CLI + verify,
- intégrer 1 workflow PR / CI,
- landing page minimaliste,
- Stripe Checkout + PayPal.

## Semaines 3–4

Objectif : **preuve d’usage**.

Faire :
- 2 à 3 démos réelles,
- 2 pilotes gratuits ou peu chers,
- obtenir des artefacts et feedbacks,
- mesurer ce que les gens valorisent : preuve, audit, replay, cross-validation, export, self-hosting.

## Semaines 5–8

Objectif : **premier revenu sérieux**.

Faire :
- vendre Agent Evidence Sprint,
- intégrer 1 ou 2 équipes,
- durcir packaging / docs / onboarding,
- améliorer outputs lisibles non techniques.

## Semaines 9–12

Objectif : **première répétabilité**.

Faire :
- standardiser les templates,
- sortir version self-hosted plus propre,
- documenter les cas d’usage,
- décider si l’abonnement est prêt.

## 13. Kill criteria / red flags

Tu dois explicitement stopper ou re-scope si :

1. le produit redevient un “framework agentique” flou ;
2. le pitch principal redevient “gouvernance MCP” sans différenciation ;
3. trop de temps est passé sur dashboard/UI avant la preuve de valeur ;
4. le coût support dépasse le revenu sur un pricing low-ticket ;
5. la proposition de valeur dépend trop d’un seul écosystème plugin ;
6. le message “proof” ne résonne pas auprès des premiers prospects.

Si le message “proof” ne résonne pas, pivoter vers :
- **secure review evidence for AI-generated PRs**,
- ou **tamper-evident PR review artifacts**,
- donc un wedge encore plus concret et étroit.

## 14. Règles de travail

### 14.1. Interdictions

Interdit de :
- proposer 10 produits en parallèle,
- relancer MetaCog comme produit principal,
- relancer un plugin consumer low-ticket comme cœur du business,
- vendre la métacognition ou le S1/S2/S3 comme bénéfice principal,
- faire semblant qu’il n’y a pas de concurrence lourde.

### 14.2. Ce qui est encouragé

Encouragé :
- un wedge très étroit mais fort,
- la preuve par démo réelle,
- le self-hosted/lightweight,
- l’export d’artefacts lisibles,
- le wording compliance/audit/proof plutôt que multi-agent/intelligence.

## 15. Sortie attendue de ta part

Quand tu exécutes ce brief, tu dois produire dans cet ordre :

1. **Décision stratégique finale en 10 lignes max**
2. **Nom + positioning + ICP**
3. **Mapping repos -> produit**
4. **Spécification MVP v1**
5. **Plan 30/60/90 jours**
6. **Pricing initial**
7. **Structure landing page**
8. **Backlog d’implémentation priorisé**
9. **Risques + mitigations**
10. **Liste explicite de ce qu’on ne fait pas**

## 16. Références de marché et vérité externe à respecter

Tu dois considérer les sources suivantes comme bases de réalité externe à ne pas ignorer :

### Concurrence / plateformes
- GitHub Enterprise AI Controls & agent control plane GA (26 fév 2026)
- GitHub docs : enterprise agent management / AI Controls / audit logs
- Microsoft Agent 365 official page
- Runlayer funding blog + homepage
- Singulr AI Agent Pulse launch
- MintMCP Trust Center / governance pages

### Claude / Anthropic
- Anthropic release notes
- Agent SDK overview
- Claude Code monitoring / OTel
- MCP donated to Agentic AI Foundation

### Paiement
- Stripe docs : PayPal activation
- Stripe docs : PayPal settlement preference
- Stripe docs : subscriptions with PayPal

### Réglementaire
- European Commission AI Act timeline

## 17. Note de prudence juridique/compliance

Ne jamais affirmer :
- “preuve juridiquement incontestable”,
- “compliance garantie”,
- “conforme AI Act / SOC2 / ISO par défaut”.

Toujours préférer :
- “artefacts de preuve vérifiables”,
- “tamper-evident”,
- “supporte les workflows d’audit et de conformité”,
- “améliore la traçabilité et l’intégrité des preuves”.

## 18. Résumé ultime

Ta mission n’est pas de faire un “meilleur plugin Claude Code”.

Ta mission est de transformer ces repos en :

> **une couche de preuve vérifiable pour agents de code**
>
> plus étroite que la gouvernance générique,
> plus vendable qu’un framework R&D,
> plus défendable qu’un simple set de guardrails,
> et assez concrète pour qu’un client paie rapidement.

---

# Références utiles

- PDF business plan initial fourni par l’utilisateur : `YGN_Business_Plan_2026.pdf`
- GitHub Enterprise AI Controls / agent control plane
- Microsoft Agent 365
- Runlayer
- Singulr AI Agent Pulse
- MintMCP Trust Center
- Anthropic Claude release notes / Agent SDK / Claude Code monitoring / MCP & AAIF
- Stripe PayPal docs
- European Commission AI Act timeline
