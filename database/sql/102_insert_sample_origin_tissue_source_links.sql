-- Link tissue sources to sample origins (one origin -> many tissue sources).
-- Resolve ids by name so this is robust to the IDENTITY values assigned when
-- the parent tables were populated. By default every tissue source is offered
-- for the animal origins; plants/fungi/bacteria have no tissue sources.
INSERT INTO sample_origin_tissue_source_links (origin_id, tissue_source_id)
SELECT o.id, t.id
FROM sample_origins o
CROSS JOIN tissue_sources t
WHERE o.name IN ('Human', 'Mouse', 'Rat', 'Dog');
