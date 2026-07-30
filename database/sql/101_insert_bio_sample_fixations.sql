INSERT INTO bio_sample_fixations (name, fixative_id) VALUES 
('Chemically Fixed', (SELECT id FROM bio_sample_fixatives WHERE name = 'Formaldehyde')),
('Chemically Fixed', (SELECT id FROM bio_sample_fixatives WHERE name = 'Glutaraldehyde')),
('Chemically Fixed', (SELECT id FROM bio_sample_fixatives WHERE name = 'Farmaldehyde-Glutaraldehyde Mixtures')),
('Chemically Fixed', (SELECT id FROM bio_sample_fixatives WHERE name = 'None')),
('Cryofixation', (SELECT id FROM bio_sample_fixatives WHERE name = 'Formaldehyde')),
('Cryofixation', (SELECT id FROM bio_sample_fixatives WHERE name = 'Glutaraldehyde')),
('Cryofixation', (SELECT id FROM bio_sample_fixatives WHERE name = 'Farmaldehyde-Glutaraldehyde Mixtures')),
('Cryofixation', (SELECT id FROM bio_sample_fixatives WHERE name = 'None')),
('Chemically and Cryofixation', (SELECT id FROM bio_sample_fixatives WHERE name = 'Formaldehyde')),
('Chemically and Cryofixation', (SELECT id FROM bio_sample_fixatives WHERE name = 'Glutaraldehyde')),
('Chemically and Cryofixation', (SELECT id FROM bio_sample_fixatives WHERE name = 'Farmaldehyde-Glutaraldehyde Mixtures')),
('Chemically and Cryofixation', (SELECT id FROM bio_sample_fixatives WHERE name = 'None')),
('None', (SELECT id FROM bio_sample_fixatives WHERE name = 'None'));
