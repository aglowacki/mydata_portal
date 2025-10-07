INSERT INTO bio_sample_fixations (name, fixative_id) VALUES 
('Chemically Fixed', (SELECT id FROM bio_sample_fixatives WHERE name = 'Formaldehyde')),
('Chemically Fixed', (SELECT id FROM bio_sample_fixatives WHERE name = 'Glutaraldehyde')),
('Chemically Fixed', (SELECT id FROM bio_sample_fixatives WHERE name = 'Farmaldehyde-Glutaraldehyde Mixtures')),
('Frozen Hydrated', (SELECT id FROM bio_sample_fixatives WHERE name = 'None')),
('Dehydrated', (SELECT id FROM bio_sample_fixatives WHERE name = 'None'));
