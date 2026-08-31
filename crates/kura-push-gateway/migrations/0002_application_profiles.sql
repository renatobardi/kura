-- The original profile names encoded APNs transport environment, not a
-- verified application identity. They therefore cannot be mapped safely to
-- either closed bundle profile. Retire the pre-profile demo authority and let
-- clients re-attest under the exact server-owned application profile.
DELETE FROM push_gateway_delegations
WHERE installation_id IN (
    SELECT id FROM push_gateway_installations
    WHERE app_profile IN ('kura-ios-production', 'kura-ios-sandbox')
);

DELETE FROM push_gateway_installations
WHERE app_profile IN ('kura-ios-production', 'kura-ios-sandbox');

ALTER TABLE push_gateway_installations
    DROP CONSTRAINT push_gateway_installations_app_profile_check;
ALTER TABLE push_gateway_installations
    ADD CONSTRAINT push_gateway_installations_app_profile_check
    CHECK (app_profile IN ('kura-ios-dogfood', 'kura-ios-app-store'));
