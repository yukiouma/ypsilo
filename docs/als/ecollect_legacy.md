# ecollect_legacy.xlsx - EDC Clinical Trial Setup Specification

## Overview

This Excel file defines a complete clinical trial study configuration for an EDC (Electronic Data Capture) system. It describes events, forms, data fields, validation rules, derivations, and code lists.

**File:** `ecollect_legacy.xlsx`
**Total Sheets:** 30

---

## Sheet Summary

| Sheet | Records | Description |
|-------|---------|-------------|
| **Events** | N | Study visits/events definitions |
| **Forms** | N | eCRF form definitions |
| **EventForm** | N | Event-to-Form assignments (visit-form mapping) |
| **DataStructure** | N | Field-level data structure with SAS naming |
| **GroupItems** | N | Item groups and field configurations |
| **CodeList** | N | Coded value lists |
| **CodeListItems** | N | Individual coded values |
| **Checks** | N | Data validation rules |
| **CheckVariables** | N | Variables referenced by checks |
| **CheckActions** | N | Actions triggered by checks |
| **Derivations** | N | Calculated field derivations |
| **DerivationVariables** | N | Variables used in derivations |
| **DerivationApplyPoints** | N | Where derivations are applied |
| **EventWorkflow** | N | Visit workflow matrix |
| **Units** | N | Unit definitions |
| **UnitGroups** | N | Unit group definitions |
| **CustomFunctions** | N | Custom validation functions |
| **AnalytesInTheStudy** | N | Lab analytes |
| **ExternalDictionary** | N | External coding dictionaries |
| **ItemAccess** | N | Field-level access control |
| **ItemGroupAccess** | N | Item group access control |
| **ECRFDraft** | N | CRF draft metadata |
| **ExternalQuestion** | N | External question configurations |
| **PDFTag** | N | PDF tagging configurations |
| **PDF** | N | PDF form configurations |
| **UnitConversions** | N | Unit conversion formulas |
| **UnitConversionDataPoints** | N | Unit conversion data points |
| **RandomizationVariableMappings** | N | Randomization mappings |
| **LabVariableMappings** | N | Lab variable mappings |
| **DisplayMode_DataType** | N | Display mode to data type mapping |

---

## Core Entity Relationships

```
Events (visits)
    └── EventForm (form assignments per event)
            └── Forms (form definitions)
                    └── GroupItems (fields/items)
                            └── DataStructure (SAS mapping)
                                    ├── CodeList ←→ CodeListItems
                                    ├── Checks ←→ CheckVariables + CheckActions
                                    └── Derivations ←→ DerivationVariables + DerivationApplyPoints
```

---

## Sheet Schemas

### Events
Defines study visits/events with timing windows.

| Field | Type | Description |
|-------|------|-------------|
| OID | String | Event identifier |
| SortNumber | Integer | Display order |
| Name | String | Event name |
| EventRepeat | Boolean | Can event repeat? |
| EventType | Enum | Scheduled, Unscheduled, Common |
| AllowedUserAdd | Boolean | Can user add? |
| AddUpperLimit | Integer | Upper limit for additions |
| Active | Boolean | Is active? |
| IsEventGroup | Boolean | Is this an event group? |
| EventGroupOID | String | Parent event group reference |
| BaselineOID | String | Baseline event reference |
| NotOpenQuery | Boolean | Disable query opening? |
| StartWin | Integer | Visit window start offset |
| Target | Integer | Target day |
| EndWin | Integer | Visit window end offset |
| Unit | Enum | Day, Week, Month |
| ViewRestrictions | String | View permission restrictions |
| AddRestrictions | String | Add permission restrictions |
| DeleteRestrictions | String | Delete permission restrictions |
| FontSize | Integer | Font size |
| FontColor | String | Font color |
| Bold | Boolean | Bold text? |
| Period | String | Study period |
| Description | String | Description |

### Forms
eCRF form definitions.

| Field | Type | Description |
|-------|------|-------------|
| OID | String | Form identifier |
| IsSubjectPage | Boolean | Is this a subject ID page? |
| Name | String | Form name |
| FormRepeat | Boolean | Can form repeat? |
| AllowedUserAdd | Boolean | Can user add? |
| AddUpperLimit | Integer | Upper limit for additions |
| ViewRestrictions | String | View permission restrictions |
| AddRestrictions | String | Add permission restrictions |
| DeleteRestrictions | String | Delete permission restrictions |
| FontSize | Integer | Font size |
| FontColor | String | Font color |
| Bold | Boolean | Bold text? |
| Description | String | Description |

### EventForm
Maps forms to events with sort order.

| Field | Type | Description |
|-------|------|-------------|
| EventOID | String | Event/visit reference |
| SortNumber | Integer | Order within visit |
| FormOID | String | Form reference |
| Active | Boolean | Is active? |
| IsNotSignatureRequired | Boolean | Signature required? |
| FormName | String | Form name |

### DataStructure
Field-level definitions with SAS dataset/field naming.

| Field | Type | Description |
|-------|------|-------------|
| FormOID | String | Parent form |
| FormName | String | Form name |
| SASDatasetName | String | SAS dataset name |
| ItemGroupRepeat | Boolean | Item group repeats? |
| ItemGroupActive | Boolean | Item group active? |
| SASFieldName | String | SAS field name |
| ItemName | String | Field question text |
| DisplayMode | Enum | RadioButton, TextField, CheckBox, DropDown, etc. |
| DataFormat | String | Data format (e.g., $200 for text) |
| CodeListOID | String | Associated code list |
| AnnotationCodeListOID | String | Annotation code list |
| UnitGroupOID | String | Associated unit group |
| QuestionTxt | String | Question text |
| DefaultValue | String | Default value |
| Active | Boolean | Is active? |
| Required | Boolean | Is field required? |
| AllowedEntry | Boolean | Is entry allowed? |
| AllowedModify | Boolean | Is modification allowed? |
| IsSDV | Boolean | Subject Data Verification required? |
| IsSDR | Boolean | Static Data Review required? |
| IsInvestigatorReview | Boolean | Investigator review required? |
| IsReview | Boolean | Review required? |
| IsMedicalReview | Boolean | Medical review required? |
| IsSafetyReview | Boolean | Safety review required? |
| IsClinicalSignificance | Boolean | Clinical significance check? |
| AsFormDate | Boolean | Use as form date? |
| AsVisitDate | Boolean | Use as visit date? |
| OtherVisits | String | Other visit references |
| IsNotSignatureRequired | Boolean | Signature required? |
| AsSubjectID | Boolean | Use as subject ID? |
| AllowedFutureDate | Boolean | Allow future dates? |
| AsDataExchageKey | Boolean | Data exchange key? |

### GroupItems
Detailed field/item configuration.

| Field | Type | Description |
|-------|------|-------------|
| FormOID | String | Parent form |
| SASDatasetName | String | SAS dataset name |
| GroupOID | String | Item group OID |
| ItemOID | String | Item OID |
| SASFieldName | String | SAS field name |
| GroupSortNumber | Integer | Group display order |
| ItemSortNumber | Integer | Item display order |
| GroupActive | Boolean | Group active? |
| ItemActive | Boolean | Item active? |
| GroupName | String | Group name |
| HiddenName | String | Hidden field name |
| ItemGroupRepeat | Boolean | Item group repeats? |
| ApplyRange | String | Apply range |
| AllowedUserAdd | Boolean | Can user add? |
| AddUpperLimit | Integer | Upper limit |
| DisplayMode | Enum | Input control type. Values: AnalytesOption, AnalytesResult, CheckBox, DateTime, DropDownList, DynamicOptions, Hidden, Label, LongText, Number, RadioButton, RadioButton(Vertical), TextField |
| DataFormat | String | Data format |
| CheckFieldDataFormat | String | Check format |
| ItemName | String | Item name |
| SASLabel | String | SAS label |
| CodeListOID | String | Code list reference |
| CodingDictionaryOID | String | Dictionary reference |
| AnnotationCodeListOID | String | Annotation code list |
| UnitGroupOID | String | Unit group reference |
| QuestionTxt | String | Question text |
| ExternalQuestion | String | External question |
| DefaultValue | String | Default value |
| CheckFieldRequired | Boolean | Check field required? |
| Required | Boolean | Required? |
| AllowedEntry | Boolean | Entry allowed? |
| AllowedModify | Boolean | Modify allowed? |
| IsSDV | Boolean | SDV required? |
| IsSDR | Boolean | SDR required? |
| IsReview | Boolean | Review required? |
| IsMedicalReview | Boolean | Medical review? |
| IsSafetyReview | Boolean | Safety review? |
| IsInvestigatorReview | Boolean | Investigator review? |
| IsClinicalSignificance | Boolean | Clinical significance? |
| AsFormDate | Boolean | As form date? |
| AsVisitDate | Boolean | As visit date? |
| OtherVisits | String | Other visits |
| IsNotSignatureRequired | Boolean | No signature? |
| AsSubjectID | Boolean | As subject ID? |
| AllowedFutureDate | Boolean | Allow future dates? |
| AsDataExchageKey | Boolean | Data exchange key? |
| ItemNameFontSize | Integer | Item name font size |
| ItemNameFontColor | String | Item name color |
| ItemNameBold | Boolean | Item name bold? |
| ItemDescription | String | Item description |
| GroupNameFontSize | Integer | Group name font size |
| GroupNameFontColor | String | Group name color |
| GroupNameBold | Boolean | Group name bold? |
| GroupDescription | String | Group description |
| ItemEnableStatus | String | Enable status |

### CodeList
Coded value lists.

| Field | Type | Description |
|-------|------|-------------|
| OID | String | Code list identifier |
| Name | String | Display name |
| CodeListType | String | Code list type |
| DataType | Enum | Integer, String, Float, Date |
| CodingType | String | Coding type |
| Annotation | String | Annotation |
| EnableStatus | String | Enable/Disable status |
| Description | String | Description |

### CodeListItems
Individual coded values.

| Field | Type | Description |
|-------|------|-------------|
| CodeListOID | String | Parent code list |
| SortNumber | Integer | Display order |
| DisplayValue | String | User-facing value |
| CodedValue | String | Stored value |
| IsUserSpecify | Boolean | User can specify? |
| CalculatedValue | String | Calculated value |

### Checks
Data validation rules.

| Field | Type | Description |
|-------|------|-------------|
| OID | String | Check identifier |
| Name | String | Check name |
| EnableStatus | String | Enable/Disable |
| ExecuteWhenMigrating | Boolean | Execute during migration? |
| Description | String | Rule description |
| AnchorDataPoint | String | Anchor reference |
| PreCondition | String | Evaluation condition |

### CheckVariables
Variables referenced by checks.

| Field | Type | Description |
|-------|------|-------------|
| CheckOID | String | Parent check |
| VariableName | String | Variable name (Var1, Var2, etc.) |
| EventOID | String | Event reference |
| FormOID | String | Form reference |
| ItemGroupOID | String | Item group reference |
| ItemOID | String | Item reference |
| RecordNo | Integer | Record number |
| PageNo | Integer | Page number |
| EventNo | Integer | Event number |
| GetValueType | String | How to get value |
| LogicalRecordPosition | Integer | Record position |
| Scope | String | Scope (Record/Form/Event) |
| SortBy | String | Sort field |
| ModifyWithoutActive | Boolean | Modify without active? |
| IsInactivedCalculate | Boolean | Inactive calculate? |

### CheckActions
Actions triggered when checks fail.

| Field | Type | Description |
|-------|------|-------------|
| CheckOID | String | Parent check |
| EventOID | String | Event reference |
| FormOID | String | Form reference |
| ItemGroupOID | String | Item group reference |
| ItemOID | String | Item reference |
| RecordNo | Integer | Record number |
| PageNo | Integer | Page number |
| EventNo | Integer | Event number |
| LogicalRecordPosition | Integer | Record position |
| Scope | String | Scope |
| SortBy | String | Sort field |
| FollowVariableName | String | Follow variable |
| ActionType | String | Action type |
| ActionString | String | Action definition |
| ActionOptions | String | Action options |
| ActionDynamicOption | String | Dynamic options |

### Derivations
Calculated field formulas.

| Field | Type | Description |
|-------|------|-------------|
| OID | String | Derivation identifier |
| Name | String | Name |
| Type | String | Compute type |
| EnableStatus | String | Enable/Disable |
| ExecuteWhenMigrating | Boolean | Execute during migration? |
| Description | String | Description |
| AnchorDataPoint | String | Anchor reference |
| ComputationalFormula | String | Formula expression |

### DerivationVariables
Variables used in derivations.

| Field | Type | Description |
|-------|------|-------------|
| DerivationOID | String | Parent derivation |
| VariableName | String | Variable name |
| EventOID | String | Event reference |
| FormOID | String | Form reference |
| ItemGroupOID | String | Item group reference |
| ItemOID | String | Item reference |
| RecordNo | Integer | Record number |
| PageNo | Integer | Page number |
| EventNo | Integer | Event number |
| GetValueType | String | How to get value |
| LogicalRecordPosition | Integer | Record position |
| Scope | String | Scope |
| SortBy | String | Sort field |
| ModifyWithoutActive | Boolean | Modify without active? |
| IsInactivedCalculate | Boolean | Inactive calculate? |

### DerivationApplyPoints
Where derivations are applied.

| Field | Type | Description |
|-------|------|-------------|
| DerivationOID | String | Parent derivation |
| EventOID | String | Event reference |
| FormOID | String | Form reference |
| ItemGroupOID | String | Item group reference |
| ItemOID | String | Item reference |
| RecordNo | Integer | Record number |
| PageNo | Integer | Page number |
| EventNo | Integer | Event number |
| LogicalRecordPosition | Integer | Record position |
| Scope | String | Scope |
| SortBy | String | Sort field |
| FollowVariableName | String | Follow variable |

### UnitConversions
Unit conversion formulas.

| Field | Type | Description |
|-------|------|-------------|
| DerivationOID | String | Parent derivation |
| OriginUnitOID | String | Source unit |
| TargetUnitOID | String | Target unit |
| ConstantA | Float | Conversion constant A |
| ConstantB | Float | Conversion constant B |
| ConstantX | Float | Conversion constant X |
| ConstantP | Float | Conversion constant P |

### Units
Unit definitions.

| Field | Type | Description |
|-------|------|-------------|
| UnitGroupOID | String | Parent unit group |
| UnitActive | Boolean | Active? |
| SortNumber | Integer | Display order |
| OID | String | Unit identifier |
| Symbol | String | Unit symbol |
| Value | Float | Numeric value |
| EnableStatus | String | Enable/Disable |

### UnitGroups
Unit group definitions.

| Field | Type | Description |
|-------|------|-------------|
| OID | String | Unit group identifier |
| Name | String | Group name |
| EnableStatus | String | Enable/Disable |

### CustomFunctions
User-defined functions for validation.

| Field | Type | Description |
|-------|------|-------------|
| FunctionName | String | Function name |
| SourceCode | String | Implementation code |
| Lang | String | Language |
| Type | String | Function type |

### AnalytesInTheStudy
Lab test definitions.

| Field | Type | Description |
|-------|------|-------------|
| AnalytesCode | String | Analyte code |
| AllowedUserSelectUnit | Boolean | User can select unit? |
| AllowedEntryLTOrGT | Boolean | Allow less than/greater than? |
| LBTESTCD | String | Lab test code |
| AnalytesName | String | Analyte name |

### ExternalDictionary
External coding dictionary references.

| Field | Type | Description |
|-------|------|-------------|
| CodeListOID | String | Code list reference |
| DictionaryName | String | Dictionary name |
| DictionaryVersion | String | Dictionary version |

### EventWorkflow
Visit workflow matrix.

| Field | Type | Description |
|-------|------|-------------|
| CRF\\Visit | String | Visit name column |
| [VisitColumns] | String | One column per scheduled visit |

### ItemAccess
Field-level access control.

| Field | Type | Description |
|-------|------|-------------|
| ItemGroupOID | String | Item group reference |
| ItemOID | String | Item reference |
| SystemType | String | System type |
| ViewRestrictions | String | View restrictions |
| ModifyRestrictions | String | Modify restrictions |

### ItemGroupAccess
Item group access control.

| Field | Type | Description |
|-------|------|-------------|
| ItemGroupOID | String | Item group reference |
| SystemType | String | System type |
| ViewRestrictions | String | View restrictions |
| AddRestrictions | String | Add restrictions |
| ModifyRestrictions | String | Modify restrictions |
| DeleteRestrictions | String | Delete restrictions |

### RandomizationVariableMappings
Randomization configuration mappings.

| Field | Type | Description |
|-------|------|-------------|
| EventOID | String | Event reference |
| FormOID | String | Form reference |
| ItemGroupOID | String | Item group reference |
| ItemOID | String | Item reference |
| MappingKey | String | Mapping key |
| LogicalPosition | Integer | Logical position |
| SortBy | String | Sort field |
| IsReferenceInfo | Boolean | Is reference info? |
| EnableStatus | String | Enable/Disable |

### LabVariableMappings
Lab variable mappings.

| Field | Type | Description |
|-------|------|-------------|
| NormalRangeOID | String | Normal range reference |
| GlobalVariableOID | String | Global variable reference |
| EventOID | String | Event reference |
| FormOID | String | Form reference |
| ItemGroupOID | String | Item group reference |
| ItemOID | String | Item reference |
| LogicalPosition | Integer | Logical position |
| SortBy | String | Sort field |

### ECRFDraft
CRF draft metadata.

| Field | Type | Description |
|-------|------|-------------|
| DraftName | String | Draft name |
| StudyName | String | Study name |
| AsNewSubjectFormOID | String | New subject form reference |
| DatabaseVersion | String | Database version |
| ProtocolVersion | String | Protocol version |
| CRFVersion | String | CRF version |
| CRFEffectiveDate | Date | CRF effective date |
| Description | String | Description |

### ExternalQuestion
External question configurations.

| Field | Type | Description |
|-------|------|-------------|
| Code | String | Question code |
| Version | String | Version |
| ExternalQuestion | String | External question definition |

### PDFTag
PDF tagging configurations.

| Field | Type | Description |
|-------|------|-------------|
| OID | String | Tag identifier |
| Content | String | Tag content |
| Language | String | Language |

### PDF
PDF form configurations.

| Field | Type | Description |
|-------|------|-------------|
| FormOID | String | Form reference |
| UseAllForms | Boolean | Use all forms? |
| PDFTagOID | String | PDF tag reference |
| OID | String | PDF identifier |
| FileRelativePath | String | File path |

### UnitConversionDataPoints
Unit conversion data points.

| Field | Type | Description |
|-------|------|-------------|
| DerivationOID | String | Derivation reference |
| ItemOID | String | Item reference |
| DefaultUnitOID | String | Default unit reference |

### DisplayMode_DataType
Display mode to data type mapping.

| Field | Type | Description |
|-------|------|-------------|
| DisplayMode | String | Display mode |
| DataType | String | Data type |
| ActionType | String | Action type |

---

## Data Flow Summary

1. **Events** - Define study timeline (visits) with scheduling windows
2. **EventForm** - Assign Forms to each Event (visit-form matrix)
3. **Forms** - Define the eCRF pages
4. **GroupItems** - Define fields within each form
5. **DataStructure** - Map fields to SAS datasets/fields with metadata
6. **CodeList + CodeListItems** - Provide valid values for fields
7. **Checks + CheckVariables + CheckActions** - Enforce data validity rules
8. **Derivations + DerivationVariables + DerivationApplyPoints** - Compute calculated fields
9. **CustomFunctions** - Provide specialized validation logic
10. **UnitConversions + UnitConversionDataPoints** - Handle unit transformations
11. **ItemAccess + ItemGroupAccess** - Control field-level permissions
12. **EventWorkflow** - Define visit workflow states
13. **AnalytesInTheStudy + LabVariableMappings** - Configure lab tests
14. **RandomizationVariableMappings** - Configure randomization
15. **ExternalDictionary** - Link to external coding dictionaries
