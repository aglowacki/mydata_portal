INSERT INTO "Users" (badge, username, user_type) VALUES 
(231160, 'aglowacki', 'Arthur', 'Glowacki', 'ANL APS XSD SDM', 'aglowacki@anl.gov' (SELECT id FROM UserTypes WHERE level = 'Admin')),
(67722, 'oantipova', 'Olga', 'Antipova', 'ANL APS XSD MIC', 'oantipova@anl.gov' (SELECT id FROM UserTypes WHERE level = 'Staff')),
(42455, 'cai', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(217631, 'sichen', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(222816, 'junjingdeng', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(40818, 'blai', 'Barry', 'Lai', 'ANL APS XSD MIC', 'blai@anl.gov' (SELECT id FROM UserTypes WHERE level = 'Staff')),
(214840, 'mfrith', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(218460, 'luxili', 'Luxi', 'Li', 'ANL APS XSD MIC', 'luxili@anl.gov', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(284110, 'yluo89', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(55523, 'qjin', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(50781, 'maser', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(53748, 'emaxey', 'Evan', 'Maxey', 'ANL APS XSD MIC', 'emaxey@anl.gov' (SELECT id FROM UserTypes WHERE level = 'Staff')),
(288366, 'swieghold', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(56129, 'vrose', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(300284, 'tzhou', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(86157, 'mvholt', (SELECT id FROM UserTypes WHERE level = 'Staff'))
;

