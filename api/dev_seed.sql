INSERT INTO users (id, google_sub, email, display_name, status)
VALUES ('00000000-0000-0000-0000-000000000001', 'dev-local', 'dev@localhost', 'Dev', 'active')
ON CONFLICT (id) DO NOTHING;
