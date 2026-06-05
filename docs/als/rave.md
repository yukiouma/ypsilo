# rave.xml — Medidata Rave EDC Excel XML Export

**File:** `.mock_data/als/rave.xml`
**Size:** 178 MB | **Lines:** 4,813,080
**Format:** Microsoft Excel SSXML (Excel 2003 XML Spreadsheet)
**App:** `<?mso-application progid="Excel.Sheet"?>`

## Overview

This is a Medidata Rave EDC (Electronic Data Capture) study definition export. It encodes a full eCRF (electronic Case Report Form) structure — the study protocol, forms, fields, visit folders, coding dictionaries, unit dictionaries, and matrix (repeating form) definitions — as an Excel XML workbook. It is the canonical interchange format for migrating Rave study designs between environments.

The file contains one **CRFDraft** (root study definition), multiple **Forms**, **Fields**, **Folders**, **DataDictionaries**, **DataDictionaryEntries**, **UnitDictionaries**, **UnitDictionaryEntries**, **Matrices**, and 40+ **Matrix\#{Cxx}** sheets (one per repeating form instance).

---

## Style System

All sheets share a common style library defined in `<Styles>`:

| Style ID | Bold | Italic | Interior | Use |
|----------|------|--------|----------|-----|
| `ColumnCaption` | ✅ | ❌ | — | Header rows |
| `ColumnCaption_alt` | ✅ | ❌ | `#ECE9D8` | Alternating header rows |
| `Default` | ❌ | ❌ | — | Data cells |
| `Default_alt` | ❌ | ❌ | `#ECE9D8` | Alternating data rows |
| `Protected` | ❌ | ❌ | — | OID/key cells (read-only) |
| `Protected_alt` | ❌ | ❌ | `#ECE9D8` | Alternating OID cells |
| `CenteredCell` | ❌ | ❌ | — | Centered data |
| `CenteredCell_alt` | ❌ | ❌ | `#ECE9D8` | Alternating centered |

Most sheets use `ColumnCaption` for header rows, `Protected` for OID/key identifier columns, and `Default` for data. The `_alt` suffix denotes an alternating row background (light beige `#ECE9D8`) used for zebra-stripe readability.

---

## Sheets

### 1. CRFDraft

**Line:** 74 | **Purpose:** Root / project-level study definition. Exactly one row.

| Column | Type | Example |
|--------|------|---------|
| `DraftName` | String | `STUDY-001_V2.0_11Jul2025` |
| `DeleteExisting` | Boolean | `FALSE` |
| `ProjectName` | String | `STUDY-001` |
| `ProjectType` | String | `Project` |
| `PrimaryFormOID` | String | `SC` — links to Forms sheet |
| `DefaultMatrixOID` | String | `DEFAULT` |
| `ConfirmationMessage` | String | `This form is submitted successfully.` |
| `SignaturePrompt` | String | `I have reviewed the data on this case report form and to the best of my knowledge it is accurate and complete.` |
| `LabStandardGroup` | String | `Standard Group_COM` |
| `ReferenceLabs` | String | `Project` |
| `AlertLabs` | Boolean | `FALSE` |
| `SyncOIDProject` | String | `akesobio2023` |
| `SyncOIDDraft` | Formula | `=IF(LEN(Forms!RC1)>0,Forms!RC1,"")` |
| `SyncOIDProjectType` | String | |
| `SyncOIDOriginIsVersion` | String | |
| `SourceUrlId` | String | |

**Hidden column 200:** Formula `=IF(LEN(Forms!RC1)>0,Forms!RC1,"")` — pulls the first Form's OID into a hidden system column.

---

### 2. Forms

**Line:** 2754 | **Purpose:** Form definitions (one row per eCRF form). Forms are the primary organizational unit in Rave — they correspond to individual data entry screens (visits, consent, AE logging, lab results, etc.).

| Column | Type | Notes |
|--------|------|-------|
| `OID` | String (PK) | Primary identifier — `SC`, `SV`, `SV_UN`, `DS_ICF`, `DM`, `AE`, `LB`, … |
| `Ordinal` | Integer | Display order in the eCRF |
| `DraftFormName` | String | Display name — `Subject Identifier`, `Visit Date`, `Unscheduled Visit`, `Informed Consent`, … |
| `DraftFormActive` | Boolean | `TRUE` / `FALSE` |
| `HelpText` | String | Inline help |
| `IsTemplate` | Boolean | `TRUE` — most forms are templates |
| `IsSignatureRequired` | Boolean | `TRUE` — requires investigator signature |
| `IsEproForm` | Boolean | `FALSE` — not an ePRO (patient-reported) form |
| `ViewRestrictions` | String | Role-based view access control |
| `EntryRestrictions` | String | Role-based entry access control |
| `LogDirection` | String | |
| `DDEOption` | String | `MayDDE` — allows Direct Data Entry |
| `ConfirmationStyle` | String | |
| `LinkFolderOID` | String | FK → Folders.OID — which visit folder this form belongs to |
| `LinkFormOID` | String | FK → Forms.OID — chained/sub-form link |
| `DownloadedFromObjectId` | String | Rave internal object ID |
| `SourceObjectId` | String | |
| `SourceUrlId` | String | |

**Hidden column 200:** Formula `=IF(LEN(Folders!RC1)>0,Folders!RC1,"")` — pulls the linked Folder OID into a hidden system column.

**Sample data:**

| OID | Ordinal | DraftFormName | DraftFormActive | IsSignatureRequired | DDEOption |
|-----|---------|---------------|-----------------|---------------------|-----------|
| SC | 1 | Subject Identifier | TRUE | TRUE | MayDDE |
| SV | 2 | Visit Date | TRUE | TRUE | MayDDE |
| SV_UN | 3 | Unscheduled Visit | TRUE | TRUE | MayDDE |
| DS_ICF | 4 | Informed Consent | TRUE | TRUE | MayDDE |
| DM | 5 | Demographics | TRUE | TRUE | MayDDE |

---

### 3. Fields

**Line:** 7524 | **Purpose:** Field definitions across all forms. This is the widest sheet (52 columns) and contains the most rows. Each row is one field on one form, with its OID, SAS variable name, data format, coding dictionary, control type, validation ranges, and metadata.

| Column | Type | Notes |
|--------|------|-------|
| `FormOID` | String | FK → Forms.OID |
| `FieldOID` | String | Primary identifier within the form — `SITEID`, `BRTHDT`, `SEX`, `AE_AESER`, `LB_LBALT`… |
| `Ordinal` | Integer | Display order on the form |
| `DraftFieldNumber` | String | Draft version field number |
| `DraftFieldName` | String | |
| `DraftFieldActive` | Boolean | |
| `VariableOID` | String | SAS variable name |
| `DataFormat` | String | Format string, e.g. `YYYY-MM-DD`, `@`, `NUMBER` |
| `DataDictionaryName` | String | FK → DataDictionaries — the code list controlling this field's values |
| `UnitDictionaryName` | String | FK → UnitDictionaries — for numeric fields with units |
| `CodingDictionary` | String | |
| `ControlType` | String | UI widget type — `Text`, `Select`, `Check`, `Radio`, `File`, … |
| `AcceptableFileExtensions` | String | For file upload fields |
| `IndentLevel` | Integer | Indentation level for grouped fields |
| `PreText` | String | Label text shown before the control |
| `FixedUnit` | String | Fixed unit label (not from dictionary) |
| `HeaderText` | String | Column group header |
| `HelpText` | String | Field-level help |
| `SourceDocument` | String | `Paper` / `EDC` / `Both` |
| `IsLog` | Boolean | Is this a log (auditable) field |
| `DefaultValue` | String | |
| `SASLabel` | String | SAS label |
| `SASFormat` | String | SAS format name |
| `EproFormat` | String | |
| `IsRequired` | Boolean | |
| `QueryFutureDate` | Boolean | |
| `IsVisible` | Boolean | |
| `IsTranslationRequired` | Boolean | |
| `AnalyteName` | String | Lab analyte name (e.g. `ALT`, `AST`, `Hemoglobin`) |
| `IsClinicalSignificance` | Boolean | Flag for clinically significant results |
| `QueryNonConformance` | Boolean | |
| `OtherVisits` | String | |
| `CanSetRecordDate` | Boolean | |
| `CanSetDataPageDate` | Boolean | |
| `CanSetInstanceDate` | Boolean | |
| `CanSetSubjectDate` | Boolean | |
| `DoesNotBreakSignature` | Boolean | |
| `LowerRange` | String | Normal range lower bound |
| `UpperRange` | String | Normal range upper bound |
| `NCLowerRange` | String | Non-clinical lower range |
| `NCUpperRange` | String | Non-clinical upper range |
| `ViewRestrictions` | String | |
| `EntryRestrictions` | String | |
| `ReviewGroups` | String | |
| `IsVisualVerify` | Boolean | |
| `FDownloadedFromObjectId` | String | Field version object ID |
| `FSourceObjectId` | String | |
| `VDownloadedFromObjectId` | String | Verification version object ID |
| `VSourceObjectId` | String | |
| `FSourceUrlId` | String | |
| `VSourceUrlId` | String | |
| `AnalyteName_ValCol` | String | |

**Hidden columns 201–202.**

---

### 4. Folders

**Line:** 105380 | **Purpose:** Visit folder / schedule definition. One row per visit (folder) in the study timeline. Folders define the scheduling windows (target day, window days) and group forms into visits.

| Column | Type | Notes |
|--------|------|-------|
| `OID` | String (PK) | Primary identifier — `C1`, `C2`, … `C1_1`, `C10`… (Cycle N) |
| `Ordinal` | Integer | |
| `FolderName` | String | Display name — `Cycle 01`, `Cycle 02`, … `Cycle 10` |
| `AccessDays` | Integer | |
| `StartWinDays` | Integer | Start of visit window (days from baseline) |
| `Targetdays` | Integer | Target visit day |
| `EndWinDays` | Integer | End of visit window |
| `OverDueDays` | Integer | Days after window end considered overdue |
| `CloseDays` | Integer | Days after window end the visit is auto-closed |
| `ParentFolderOID` | String | FK → Folders.OID — for nested/sub-folder hierarchy |
| `IsReusable` | Boolean | `FALSE` — folders are not typically reusable in cycles |
| `DownloadedFromObjectId` | String | |
| `SourceObjectId` | String | |
| `SourceUrlId` | String | |

**Hidden columns 11–13.**

---

### 5. DataDictionaries

**Line:** 106974 | **Purpose:** Controlled terminology (code list) definitions. Each DataDictionary is a collection of coded values for a field (e.g. adverse event causality, outcome, severity, drug action taken).

| Column | Type |
|--------|------|
| `DataDictionaryName` | String (PK) |
| `OID` | String |
| `DownloadedFromObjectId` | String |
| `SourceObjectId` | String |
| `SourceUrlId` | String |

**Examples:** `AE_AEACN_CM` (AE action taken), `AE_AEOUT` (AE outcome), `AE_AEREL` (AE relatedness), `AE_AETOXGR` (AE toxicity grade), `AE_OC_AEACN_AK` (AK-specific AE action).

---

### 6. DataDictionaryEntries

**Line:** 107637 | **Purpose:** Individual coded values within a DataDictionary. One row per code-value pair.

| Column | Type | Notes |
|--------|------|-------|
| `DataDictionaryName` | String | FK → DataDictionaries |
| `CodedData` | String | Code — `1`, `2`, `3`… |
| `Ordinal` | Integer | Display/sort order |
| `UserDataString` | String | Display text — `No Action Taken`, `Interrupted`, `Dose Reduced`, `Discontinued Permanently`… |
| `Specify` | Boolean | `TRUE` = allows free-text "specify" when this code is selected |

**Hidden column 200:** Formula `=IF(LEN(DataDictionaries!RC1)>0,DataDictionaries!RC1,"")` — pulls parent DataDictionary name.

**Sample — AE_AEACN_CM:**

| CodedData | Ordinal | UserDataString | Specify |
|-----------|---------|---------------|---------|
| 1 | 1 | No Action Taken | FALSE |
| 2 | 2 | Interrupted | FALSE |
| 3 | 3 | Dose Reduced | FALSE |
| 4 | 4 | Discontinued Permanently | FALSE |
| 5 | 5 | Dose Discontinued | FALSE |
| 6 | 6 | Unknown | FALSE |

---

### 7. UnitDictionaries

**Line:** 118827 | **Purpose:** Unit of measure definitions for numeric/lab fields.

| Column | Type |
|--------|------|
| `UnitDictionaryName` | String (PK) |
| `StandardUnitName` | String |
| `OID` | String |
| `DownloadedFromObjectId` | String |
| `SourceObjectId` | String |
| `SourceUrlId` | String |

**Hidden column 200:** pulls `UnitDictionaryEntries!RC8`.

---

### 8. UnitDictionaryEntries

**Line:** 119033 | **Purpose:** Individual unit values within a UnitDictionary. Includes conversion formula constants (A, B, C, K) for unit conversion.

| Column | Type | Notes |
|--------|------|-------|
| `UnitDictionaryName` | String | FK → UnitDictionaries |
| `CodedUnit` | String | Code |
| `Ordinal` | Integer | |
| `ConstantA` | Float | Conversion constant A (y = Ax + B for unit conversion) |
| `ConstantB` | Float | Conversion constant B |
| `ConstantC` | Float | Conversion constant C |
| `ConstantK` | Float | Conversion constant K |
| `UnitString` | String | Display — `mg`, `kg`, `mmol/L`, `g/L`… |

**Hidden column 200:** Formula `=IF(LEN(UnitDictionaries!RC1)>0,UnitDictionaries!RC1,"")` — pulls parent UnitDictionary name.

---

### 9. Matrices

**Line:** 119403 | **Purpose:** Repeating form group definitions. A Matrix is a form that can repeat N times (e.g. lab results for each cycle of treatment). Each Matrix has a `Maximum` instances setting.

| Column | Type | Notes |
|--------|------|-------|
| `MatrixName` | String | Display name — `Cycle 01`, `Cycle 01_1`, `Cycle 10`… |
| `OID` | String (PK) | Primary identifier — `C1`, `C1_1`, `C10`… |
| `Addable` | Boolean | `FALSE` — cannot manually add additional instances beyond Maximum |
| `Maximum` | Integer | Maximum allowed instances (0 = unbounded within form limits) |
| `DownloadedFromObjectId` | String | |
| `SourceObjectId` | String | |
| `SourceUrlId` | String | |

**Hidden columns 4–6.**

---

### 10. Matrix1#C1 … MatrixN#Cxx (repeating forms)

**Line:** 121254+ | **Purpose:** Individual repeating form data entry grids. Each Matrix sheet represents one Matrix OID's form structure with discrete timepoint columns.

The sheet name follows `Matrix{MatrixNum}#{FolderOID}`:
- `Matrix1#C1` — Matrix 1, Folder version C1
- `Matrix2#C11` — Matrix 2, Folder version C11
- `Matrix3#C10` — Matrix 3, Folder version C10

The `#Cxx` suffix encodes both the cycle and the version/form variant of that matrix.

| Column pattern | Type | Notes |
|---------------|------|-------|
| `Matrix: {OID}` | String | Row header — e.g. `Matrix: C1` |
| `Subject` | String | Subject/site identifier |
| `SCR` | String | Screening visit column |
| `C1` | String | Cycle 1 timepoint |
| `C2` | String | Cycle 2 timepoint |
| … | | |
| `C41` | String | Cycle 41 timepoint (max observed) |

Each `C1`–`C41` column represents a discrete timepoint (visit window) within that cycle. Forms linked to this matrix will repeat one row per instance.

---

## Cross-Sheet Relationships

```
CRFDraft (root — single row)
  └── PrimaryFormOID = "SC"
         │
         └── Forms (FormOID = "SC", "SV", "SV_UN", "DM", "AE", "LB", ...)
               │
               ├── LinkFolderOID ──────────────────→ Folders (FolderOID)
               │                                          │
               │                                          └── ParentFolderOID (tree)
               │
               ├── LinkFormOID ──────────────────────→ Forms (self-reference, chained forms)
               │
               └── FormOID ──────────────────────────→ Fields (FormOID)
                          │                              │
                          │                              ├── DataDictionaryName ──→ DataDictionaries
                          │                              │                              │
                          │                              │                              └── DataDictionaryEntries (CodedData, UserDataString)
                          │                              │
                          │                              └── UnitDictionaryName ──────→ UnitDictionaries
                          │                                                             │
                          │                                                             └── UnitDictionaryEntries (CodedUnit, UnitString)
                          │
                          └── Matrices (MatrixName, OID = "C1", "C1_1", "C10"...)
                                     │
                                     └── Matrix1#C1, Matrix2#C11, Matrix3#C10, ...
```

---

## Key Design Patterns

### 1. OID as Primary Key
Every entity (Forms, Folders, Fields, DataDictionaries, Matrices) uses a string `OID` as its primary key. These OIDs are the stable, canonical identifiers used across all cross-references.

### 2. Hierarchical Folder Structure
Folders form a tree via `ParentFolderOID`. The root folders are `C1`, `C2`… representing study cycles. Sub-folders use `_` suffix notation (`C1_1`, `C10_1`).

### 3. Form-to-Folder Binding
Forms are assigned to a Folder via `LinkFolderOID`. This determines which visit(s) a form appears in.

### 4. Controlled Terminology via DataDictionaries
Fields that need controlled vocabularies (AE codes, drug causality, lab units) reference a `DataDictionaryName`. The actual codes are in `DataDictionaryEntries`. The `Specify` column on each entry allows free-text when the coded value isn't sufficient.

### 5. Unit Conversion Constants
UnitDictionaryEntries store conversion formula constants (A, B, C, K) for converting between unit systems. The formula `y = Ax + B` (with K as a scaling factor) allows the system to convert values between standard and local units.

### 6. Matrix Versioning via Sheet Suffix
Each Matrix has multiple physical sheet instances (`Matrix1#C1`, `Matrix1#C11`, `Matrix1#C101`). The `#Cxx` suffix in the sheet name encodes the Folder OID (cycle) and version, so the same Matrix definition can produce different form instances per cycle.

### 7. Hidden Cross-Reference Columns
Every data sheet has hidden columns (200+) containing formulas that pull the parent entity's OID from the referenced sheet. This provides a self-contained lookup mechanism independent of row position.

### 8. Alternating Row Styles
All sheets use `_alt` style variants for every other row, applying a light beige (`#ECE9D8`) background for visual striping in Excel.

### 9. OID Columns are Protected
The `OID` column in Forms and the `FieldOID` / `FormOID` columns in Fields use `Protected` style, marking them as read-only in the Excel UI.

---

## Sheet Index

| Sheet | Line | Rows | Key Columns |
|-------|------|------|-------------|
| CRFDraft | 74 | 1 | DraftName, ProjectName, PrimaryFormOID |
| Forms | 2754 | ~dozens | OID, DraftFormName, LinkFolderOID |
| Fields | 7524 | ~hundreds | FormOID, FieldOID, VariableOID, DataDictionaryName |
| Folders | 105380 | ~dozens | OID, FolderName, Targetdays, ParentFolderOID |
| DataDictionaries | 106974 | ~dozens | DataDictionaryName, OID |
| DataDictionaryEntries | 107637 | ~hundreds | DataDictionaryName, CodedData, UserDataString |
| UnitDictionaries | 118827 | ~dozens | UnitDictionaryName, StandardUnitName |
| UnitDictionaryEntries | 119033 | ~hundreds | UnitDictionaryName, CodedUnit, UnitString |
| Matrices | 119403 | ~dozens | MatrixName, OID, Maximum |
| Matrix1#C1 … Matrix42#C281 | 121254+ | ~varies | Matrix: C1, Subject, SCR, C1–C41 |