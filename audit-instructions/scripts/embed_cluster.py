#!/usr/bin/env python3
"""
embed_cluster.py – embeddingy těl metod + clustering → out/clusters.json (vstup pro kolo 3)

Embedding model: cokoli s code-tuned vektory (Gemini embedding, voyage-code, jina-code).
Default: Gemini embedding přes google-genai (stejný klíč jako run_gemini.py).

Postup:
  1. vezme ze symbols.sqlite všechny method/function s body_tokens >= 40
  2. embedding každého těla (batch po 100, cache na disk – přerušení = pokračování)
  3. ANN (hnswlib nebo sklearn NearestNeighbors) – pro každý symbol top-k sousedů
  4. spojí do skupin: hrana když cosine >= THRESH; connected components; max velikost 30 (větší se dělí)
  5. zapíše clusters.json + do symbol.cluster_id

Použití:
  python3 embed_cluster.py --db out/symbols.sqlite --thresh 0.82 --k 15
"""
import argparse, json, os, sqlite3, pathlib, sys, time
import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[1]

def embed_gemini(texts, model="gemini-embedding-001"):
    from google import genai
    from google.genai import types
    client = genai.Client()
    out = []
    for i in range(0, len(texts), 100):
        chunk = texts[i:i+100]
        for attempt in range(5):
            try:
                r = client.models.embed_content(model=model, contents=chunk,
                        config=types.EmbedContentConfig(task_type="CODE_RETRIEVAL_QUERY"))
                out += [e.values for e in r.embeddings]; break
            except Exception as e:
                print(f"embed retry {attempt}: {e}", file=sys.stderr); time.sleep(10 * (attempt + 1))
        else:
            sys.exit("embedding failed repeatedly")
    return np.array(out, dtype=np.float32)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", default=str(ROOT / "out/symbols.sqlite"))
    ap.add_argument("--thresh", type=float, default=0.82)
    ap.add_argument("--k", type=int, default=15)
    ap.add_argument("--min-tokens", type=int, default=40)
    ap.add_argument("--max-cluster", type=int, default=30)
    a = ap.parse_args()

    db = sqlite3.connect(a.db)
    rows = db.execute("SELECT id, fqn, file, signature, body FROM symbol WHERE kind IN ('method','function') AND body_tokens >= ?", (a.min_tokens,)).fetchall()
    print(f"symbols to embed: {len(rows)}", file=sys.stderr)

    cache = pathlib.Path(a.db).parent / "embeddings.npy"
    ids_cache = pathlib.Path(a.db).parent / "embeddings_ids.json"
    if cache.exists() and ids_cache.exists() and json.loads(ids_cache.read_text()) == [r[0] for r in rows]:
        X = np.load(cache)
    else:
        texts = [f"{sig}\n{body}"[:8000] for (_, _, _, sig, body) in rows]
        X = embed_gemini(texts)
        np.save(cache, X); ids_cache.write_text(json.dumps([r[0] for r in rows]))
    X /= np.linalg.norm(X, axis=1, keepdims=True) + 1e-9

    try:
        import hnswlib
        idx = hnswlib.Index(space="cosine", dim=X.shape[1]); idx.init_index(max_elements=len(X), ef_construction=200, M=32)
        idx.add_items(X, np.arange(len(X))); idx.set_ef(100)
        nbr, dist = idx.knn_query(X, k=a.k + 1); sim = 1 - dist
    except ImportError:
        from sklearn.neighbors import NearestNeighbors
        nn = NearestNeighbors(n_neighbors=a.k + 1, metric="cosine").fit(X)
        dist, nbr = nn.kneighbors(X); sim = 1 - dist

    # union-find nad hranami >= thresh
    parent = list(range(len(X)))
    def find(i):
        while parent[i] != i: parent[i] = parent[parent[i]]; i = parent[i]
        return i
    def union(i, j):
        ri, rj = find(i), find(j)
        if ri != rj: parent[ri] = rj
    edges = []
    for i in range(len(X)):
        for j, s in zip(nbr[i][1:], sim[i][1:]):
            if s >= a.thresh and rows[i][2] != rows[j][2] or (s >= a.thresh and rows[i][0] != rows[j][0]):
                union(i, int(j)); edges.append((i, int(j), float(s)))

    groups = {}
    for i in range(len(X)): groups.setdefault(find(i), []).append(i)
    clusters = []
    for gid, members in groups.items():
        if len(members) < 2: continue
        # velké komponenty rozsekat podle nejsilnějších hran (hrubě: seřadit a krájet)
        for part in [members[i:i + a.max_cluster] for i in range(0, len(members), a.max_cluster)]:
            cid = f"c_{len(clusters):05d}"
            clusters.append({"cluster_id": cid, "size": len(part),
                             "members": [{"id": rows[m][0], "fqn": rows[m][1], "file": rows[m][2]} for m in part],
                             "edges": [(rows[i][1], rows[j][1], round(s, 3)) for (i, j, s) in edges if i in part and j in part]})
            db.executemany("UPDATE symbol SET cluster_id=? WHERE id=?", [(cid, rows[m][0]) for m in part])
    db.commit()
    singletons = [rows[i][0] for i in range(len(X)) if len(groups[find(i)]) == 1]
    out = {"thresh": a.thresh, "k": a.k, "clusters": clusters, "singleton_ids": singletons}
    (pathlib.Path(a.db).parent / "clusters.json").write_text(json.dumps(out, ensure_ascii=False, indent=1))
    print(f"clusters={len(clusters)} singletons={len(singletons)} edges={len(edges)}", file=sys.stderr)

if __name__ == "__main__":
    main()
