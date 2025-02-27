INSERT INTO "Users" (badge, username, first_name, last_name, institution, email, user_type) VALUES 
(231160, 'aglowacki', 'Arthur', 'Glowacki', 'ANL APS XSD SDM', 'aglowacki@anl.gov' (SELECT id FROM UserTypes WHERE level = 'Admin')),
(67722, 'oantipova', 'Olga', 'Antipova', 'ANL APS XSD MIC', 'oantipova@anl.gov' (SELECT id FROM UserTypes WHERE level = 'Staff')),
(42455, 'cai', 'Zhonghou', 'Cai', 'ANL APS XSD MIC', 'cai@anl.gov', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(217631, 'sichen', 'Si', 'Chen', 'ANL APS XSD MIC', 'sichen@anl.gov', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(222816, 'junjingdeng', 'Junjing', 'Deng', 'ANL APS XSD MIC', 'junjingdeng@anl.gov', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(40818, 'blai', 'Barry', 'Lai', 'ANL APS XSD MIC', 'blai@anl.gov' (SELECT id FROM UserTypes WHERE level = 'Staff')),
(218460, 'luxili', 'Luxi', 'Li', 'ANL APS XSD MIC', 'luxili@anl.gov', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(284110, 'yluo89', 'Yanqi', 'Luo', 'ANL APS XSD MIC', 'yluo89@anl.gov', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(55523, 'qjin', 'Qiaoling', 'Jin', 'ANL APS', 'qjin@anl.gov', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(50781, 'maser', 'Jorg', 'Maser', 'ANL APS XSD MIC', 'maser@anl.gov', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(53748, 'emaxey', 'Evan', 'Maxey', 'ANL APS XSD MIC', 'emaxey@anl.gov' (SELECT id FROM UserTypes WHERE level = 'Staff')),
(288366, 'swieghold', '', 'Wieghold', 'ANL APS XSD MIC', 'SWIEGHOLD@ANL.GOV', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(56129, 'vrose', 'Volker', 'Rose', 'ANL APS XSD MIC', 'VROSE@anl.gov', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(300284, 'tzhou', 'Tao', 'Zhou', 'ANL APS', 'TZHOU@ANL.GOV', (SELECT id FROM UserTypes WHERE level = 'Staff')),
(86157, 'mvholt', 'Martin', 'Holt', 'ANL CNM', 'MVHOLT@ANL.GOV', (SELECT id FROM UserTypes WHERE level = 'Staff'))
;

