# ecollect_v6.xlsx - EDC Clinical Trial Setup Instrument Structure

This Excel file is used to set up a clinical trial project in an EDC (Electronic Data Capture) system. It contains **28 sheets** that define the complete structure of a clinical study.

---

## Overview

| Category | Sheets | Description |
|----------|--------|-------------|
| **Reference Data** | 4 | UnitGroups, Units, CodeList, CodeListItems, MedicalDictionary |
| **Study Configuration** | 1 | ECRFDraft |
| **Form Structure** | 5 | FormSets, Forms, Items, FormItem, PDF |
| **Lab & Randomization** | 2 | AnalytesInTheStudy, LabVariableMappings, RandomizationVariableMappings |
| **Validation & Logic** | 5 | Checks, CheckVariables, CheckActions, Derivations, DerivationVariables, DerivationApplyPoints |
| **Advanced** | 1 | AdvancedProgrammings |
| **Visit Plans** | 7 | Plans, PlanSCR, PlanCYCLE, PlanEARLY, PlanCOM, PlanDSEOS, PlanUNS |

---

## Sheet Details

### 1. Reference Data

#### UnitGroups (3 cols × 19 rows)
Defines unit groups for measurement values.
- `UnitGroupOID` - Unique identifier for the unit group
- `Name` - Display name
- `Status` - Active/Inactive status

#### Units (6 cols × 19 rows)
Defines individual units within groups.
- `UnitGroupOID` - Parent unit group reference
- `UnitOID` - Unique identifier
- `UnitName` - Display name
- `Value` - Numeric value
- `IsStandard` - Whether this is a standard unit
- `ConversionFormula` - Formula for conversion

#### CodeList (5 cols × 50 rows)
Defines coded value lists for dropdowns/radio buttons.
- `OID` - Unique identifier
- `Name` - Display name
- `Annotation` - Additional notes
- `DataType` - Data type (text, number, etc.)
- `Status` - Active/Inactive

#### CodeListItems (5 cols × 297 rows)
Individual items within code lists.
- `CodeListOID` - Parent code list reference
- `DisplayValue` - What user sees
- `CodedValue` - Internal value
- `IsUserSpecify` - Allow user-defined values
- `CalculatedValue` - Computed value

#### MedicalDictionary (2 cols × 1 row)
Medical dictionary configuration.
- `DictionaryType` - e.g., MedDRA
- `DictionaryVersion` - Version number

---

### 2. Study Configuration

#### ECRFDraft (6 cols × 2 rows)
Main study configuration.
- `DraftName` - Name of the CRF draft
- `StudyName` - Clinical study name
- `AsNewSubjectFormOID` - Form for new subject registration
- `DefaultPlanOID` - Default visit plan
- `DatabaseVersion` - Database version
- `ECRFEffectiveDate` - Effective date

---

### 3. Form Structure

#### FormSets (13 cols × 18 rows)
Groups forms into logical sets (e.g., by visit).
- `FormsetOID` - Unique identifier
- `FormsetName` - Display name
- `FormsetType` - Type classification
- `IsVisible` - Visibility flag
- `Addable` - Can new records be added
- `Max` - Maximum records
- `ParentFormsetOID` - Parent formset for nesting
- `BaselineFormsetOID` - Baseline reference
- `Offset`, `Unit`, `Minus`, `Plus` - Scheduling parameters
- `GenerateOverdueQuery` - Auto-generate overdue queries

#### Forms (14 cols × 40 rows)
Individual eCRF forms.
- `FormOID` - Unique identifier
- `FormName` - Display name
- `SASDatasetName` - SAS dataset name
- `FormType` - Type classification
- `IsVisible` - Visibility
- `Except` - Exceptions
- `Addable`, `Max` - Record constraints
- `ViewRestriction`, `AddRestriction`, `DeleteRestriction` - Access control
- `Redirection`, `RedirectedFormsetOID`, `RedirectedFormOID` - Redirection settings

#### Items (14 cols × 273 rows)
Form field definitions.
- `ItemOID` - Unique identifier
- `SASFieldName` - SAS field name
- `ItemName` - Display name
- `SASLabel` - SAS label
- `ControlType` - Input control type. Possible values: `Calendar`, `Drop-down List`, `Dynamic Options`, `Lab Result`, `Lab Test`, `Radio(horizontal)`, `Radio(vertical)`, `Tags`, `Textbox`
- `DataFormat` - Data format specification
- `UkPart` - UK-specific configuration
- `CodeListOID` - Associated code list
- `MedicalDictionaryType` - Dictionary type
- `FlagOID` - Associated flag
- `FlagOpenQuery` - Open query behavior
- `UnitGroupOID` - Unit group for measurements
- `Prompt` - Field prompt text
- `CheckFormatMismatch` - Format validation

#### FormItem (53 cols × 329 rows)
Links forms to items with form-specific configuration. This is the main junction table.
- `FormOID`, `FormName` - Form reference
- `ItemOID` - Item reference
- `ItemType` - Item classification
- `IsGridVisible` - Grid visibility
- `AllowDeleteDefaultRecord` - Delete permission
- `Addable`, `Max` - Record constraints
- `IsRestrictAllRolesEdit` - Role-based edit restriction
- `CheckItemMissing` - Missing check
- `IsVisible` - Visibility
- `NormalRangeExceptionalDisplay` - Range display
- `AllowFutureDate` - Date validation
- `AsVisitDate`, `AsFormDate` - Date associations
- `DefaultValue` - Default value
- `RestrictDefaultValueEdit` - Restrict default editing
- `GeneralDefaultValue` - General default
- `AsSubjectID` - Subject ID flag
- `RequiresSDV` - Source Data Verification flag
- `RequiresDMReview`, `RequiresMedicalReview`, `RequiresSafetyReview` - Review flags
- `RequiresReview4/5/6` - Additional review flags
- `RequiresSignature` - Signature requirement
- `IsAutoCreateClinicalSignificance` - Auto-create flag
- `Color`, `FontSize`, `Bold`, `IndentLevel` - Display styling
- `ItemViewRestriction`, `ItemAddRestriction`, `ItemDeleteRestriction`, `ItemEditRestriction` - Item-level access
- `RecordViewRestriction`, `RecordAddRestriction`, `RecordDeleteRestriction`, `RecordEditRestriction` - Record-level access
- Fields 40-52 repeat item properties (SASFieldName, ItemName, etc.)

#### PDF (4 cols × 1 row)
PDF configuration.
- `PdfName` - PDF file name
- `Description` - Description
- `Language` - Language
- `AttachTo` - Attachment target

---

### 4. Lab & Randomization

#### AnalytesInTheStudy (2 cols × 47 rows)
Lab analytes.
- `AnalytesCode` - Code
- `AnalytesName` - Name

#### LabVariableMappings (6 cols × 3 rows)
Maps lab variables to global variables.
- `NormalRangeOID` - Normal range ID
- `GlobalVariableOID` - Global variable
- `FormsetOID`, `FormOID`, `ItemOID` - Target fields
- `LogicalPosition` - Position

#### RandomizationVariableMappings (7 cols × 1 row)
Randomization variable mappings.
- `Type` - Mapping type
- `MappingValue` - Value
- `FormsetOID`, `FormOID`, `ItemOID` - Target fields
- `LogicalRecordPosition` - Record position
- `OrderBy` - Ordering

---

### 5. Validation & Logic

#### Checks (4 cols × 187 rows)
Edit check definitions.
- `OID` - Unique identifier
- `Name` - Check name
- `EnableStatus` - Enabled/disabled
- `PreCondition` - Execution condition

#### CheckVariables (14 cols × 205 rows)
Variables used in checks.
- `CheckOID` - Parent check
- `VariableName` - Variable name
- `FormsetOID`, `FormOID`, `ItemOID` - Location
- `RecordNo`, `PageNo`, `FormsetNo` - Position
- `Anchor` - Anchor reference
- `LogicalRecordPosition` - Record position
- `Scope` - Variable scope
- `OrderBy` - Ordering
- `ValueType` - Data type
- `ModifyTrigger` - Modification trigger

#### CheckActions (14 cols × 359 rows)
Actions taken when checks fail.
- `CheckOID` - Parent check
- `FormsetOID`, `FormOID`, `ItemOID` - Target location
- `RecordNo`, `PageNo`, `FormsetNo` - Position
- `LogicalRecordPosition` - Record position
- `Scope` - Action scope
- `OrderBy` - Ordering
- `ActionType` - Type (message, query, etc.)
- `ActionString` - Action content
- `ActionOptions` - Options
- `ActionDynamicOption` - Dynamic options

#### Derivations (4 cols × 10 rows)
Data derivation rules.
- `OID` - Unique identifier
- `Name` - Derivation name
- `EnableStatus` - Enabled/disabled
- `ComputationalFormula` - Formula

#### DerivationVariables (14 cols × 18 rows)
Variables used in derivations.
- `DerivationOID` - Parent derivation
- `VariableName` - Variable name
- `FormsetOID`, `FormOID`, `ItemOID` - Location
- `RecordNo`, `PageNo`, `FormsetNo` - Position
- `Anchor` - Anchor reference
- `LogicalRecordPosition` - Record position
- `Scope` - Variable scope
- `OrderBy` - Ordering
- `ValueType` - Data type
- `ModifyTrigger` - Modification trigger

#### DerivationApplyPoints (11 cols × 10 rows)
When derivations are applied.
- `DerivationOID` - Parent derivation
- `FormsetOID`, `FormOID`, `ItemOID` - Target location
- `RecordNo`, `PageNo`, `FormsetNo` - Position
- `LogicalRecordPosition` - Record position
- `Scope` - Scope
- `OrderBy` - Ordering
- `ClearDataWhenError` - Clear data flag

---

### 6. Advanced

#### AdvancedProgrammings (4 cols × 1 row)
Custom programming scripts.
- `Name` - Script name
- `Description` - Description
- `Language` - Programming language
- `Sourcecode` - The code

---

### 7. Visit Plans

#### Plans (3 cols × 7 rows)
Visit plan definitions.
- `PlanName` - Plan name
- `OID` - Unique identifier
- `AllowManualActivation` - Manual activation flag

#### PlanSCR, PlanCYCLE, PlanEARLY, PlanCOM, PlanDSEOS, PlanUNS (200 cols × 40 rows each)
Visit schedule matrices showing which forms are scheduled at which visits.
- Columns: Form\Visit, SUBJECT, SCR, V1, D1, D2, D4, D8, D15, D22, D29, D43, D57, D71, D85, EARLY, COM, DSEOS, UNS
- Rows represent forms
- Cell values indicate scheduling information

---

## Relationships

```
ECRFDraft (Study Level)
    ↓
FormSets → Forms → FormItem ← Items
    ↓           ↓
    → Checks → CheckVariables
               CheckActions
    ↓
    → Derivations → DerivationVariables
                   DerivationApplyPoints
    ↓
    → Plans (PlanSCR, PlanCYCLE, PlanEARLY, PlanCOM, PlanDSEOS, PlanUNS)
```

---

## Key Fields Reference

### OID Patterns
- `UnitGroupOID` → Units.UnitGroupOID
- `CodeListOID` → CodeListItems.CodeListOID, Items.CodeListOID, FormItem.CodeListOID
- `FormsetOID` → Forms.ParentFormsetOID (optional), CheckVariables.FormsetOID, etc.
- `FormOID` → FormItem.FormOID, CheckVariables.FormOID, etc.
- `ItemOID` → FormItem.ItemOID, CheckVariables.ItemOID, etc.
- `CheckOID` → CheckVariables.CheckOID, CheckActions.CheckOID
- `DerivationOID` → DerivationVariables.DerivationOID, DerivationApplyPoints.DerivationOID

---

## Summary Statistics

| Sheet | Rows | Purpose |
|-------|------|---------|
| UnitGroups | 19 | Unit definitions |
| Units | 19 | Unit members |
| CodeList | 50 | Codelist definitions |
| CodeListItems | 297 | Codelist values |
| Forms | 40 | eCRF forms |
| Items | 273 | Form fields |
| FormItem | 329 | Form-Item mappings |
| Checks | 187 | Edit checks |
| CheckActions | 359 | Check actions |
| Plans | 7 | Visit plans |
| Plan* sheets | 40 each | Visit schedules |