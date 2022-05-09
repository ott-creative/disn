use super::VerifiableCredential;
use crate::api::PartyResult;
use chrono::{Duration, SecondsFormat, TimeZone, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

const MAX_EXPIRY_DAYS: i64 = 180;

pub struct CredentialPersonalIdentity {
    pub document_number: String,
    pub document_type: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub sex: String,
    pub dob: String,
    pub address: String,
    pub issuer_org_full: String,
    pub issuer_org_iso2: String,
    pub issuer_org_iso3: String,
    pub nationality_full: String,
    pub nationality_iso2: String,
    pub nationality_iso3: String,
    pub expiry: String,
    pub issued: String,
    pub issue_authority: String,
    pub face_is_identical: bool,
    pub face_confidence_score: String,
    pub verification_face_result: bool,
    pub verification_digit_result: bool,
    /*pub authentication_score: String,
    pub res_document_front: String,
    pub res_document_back: String,
    pub res_face_image: String,
    pub res_face_video: String,*/
}

impl VerifiableCredential for CredentialPersonalIdentity {
    fn generate_unsigned(&self, issuer: &str, holder: &str) -> Value {
        json!({
            "@context": ["https://www.w3.org/2018/credentials/v1", "https://credential.codegene.xyz/context/credential.jsonld"],
            "id": format!("urn:uuid:{}", Uuid::new_v4().to_string()),
            "type": ["VerifiableCredential"],
            "issuer": issuer,
            "holder": holder,
            "expirationDate": convert_expiry(&self.expiry),
            "credentialSubject": {
                "id": holder,
                "documentNumber": self.document_number,
                "firstName": self.first_name,
                "lastName": self.last_name,
                "fullName": self.full_name,
                "sex": self.sex,
                "dob": self.dob,
                "address1": self.address,
                "documentType": self.document_type,
                "issuerOrgFull": self.issuer_org_full,
                "issuerOrgIso2": self.issuer_org_iso2,
                "issuerOrgIso3": self.issuer_org_iso3,
                "nationalityFull": self.nationality_full,
                "nationalityIso2": self.nationality_iso2,
                "nationalityIso3": self.nationality_iso3,
                "expiry": self.expiry,
                "issued": self.issued,
                "issueAuthority": self.issue_authority,
                "faceIsIdentical": self.face_is_identical,
                "faceConfidence": self.face_confidence_score,
                "verificationFace": self.verification_face_result,
                "verificationDigit": self.verification_digit_result,
            }
        })
    }
}

impl From<PartyResult> for CredentialPersonalIdentity {
    fn from(party: PartyResult) -> Self {
        CredentialPersonalIdentity {
            document_number: party.result.documentNumber,
            document_type: party.result.documentType,
            first_name: party.result.firstName,
            last_name: party.result.lastName,
            full_name: party.result.fullName,
            sex: party.result.sex,
            dob: party.result.dob,
            address: party.result.address1,
            issuer_org_full: party.result.issuerOrg_full,
            issuer_org_iso2: party.result.issuerOrg_iso2,
            issuer_org_iso3: party.result.issuerOrg_iso3,
            nationality_full: party.result.nationality_full,
            nationality_iso2: party.result.nationality_iso2,
            nationality_iso3: party.result.nationality_iso3,
            expiry: party.result.expiry,
            issued: party.result.issued,
            issue_authority: party.result.issueAuthority,
            face_is_identical: party.face.isIdentical,
            face_confidence_score: party.face.confidence,
            verification_face_result: party.verification.result.face,
            verification_digit_result: party.verification.result.checkdigit,
        }
    }
}

fn convert_expiry(expiry: &str) -> String {
    let utc = Utc;
    let d1 = Utc::now();
    let d2 = utc
        .datetime_from_str(&format!("{} 00:00:00", expiry), "%Y/%m/%d %H:%M:%S")
        .unwrap();
    let duration = d2.signed_duration_since(d1);

    if duration.num_days() > MAX_EXPIRY_DAYS {
        return (Utc::now() + Duration::days(MAX_EXPIRY_DAYS))
            .to_rfc3339_opts(SecondsFormat::Secs, true);
    } else {
        return d2.to_rfc3339_opts(SecondsFormat::Secs, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_expiry() {
        // in MAX_EXPIRY_DAYS
        let expiry = "2022/09/28";
        let converted = convert_expiry(expiry);

        assert_eq!(converted, "2022-09-28T00:00:00Z");

        // over MAX_EXPIRY_DAYS
        let expiry = "2024/09/28";
        let converted = convert_expiry(expiry);

        let expected = (Utc::now() + Duration::days(MAX_EXPIRY_DAYS))
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(converted, expected);
    }
}
