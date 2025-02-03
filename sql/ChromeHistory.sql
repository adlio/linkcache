SELECT
    v.*,
    u.*

FROM visits v
INNER JOIN urls u ON u.id = v.url