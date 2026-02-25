
## Directory Details

### `entry/src/main`
      
#### `ets/`

- **components/**  
  Stores reusable components. The data comes from components in the design draft.

- **pages/**  
  Stores page components, with each file representing a page.

- **entity/**  
  Stores type definitions, such as interfaces for parameters passed in component references.

- **router/**  
  Stores routing data, used in scenarios like page navigation.

- **style/**  
  Stores style data, sourced from the design draft.

- **utils/**  
  Stores utility classes and screen adaptation classes.

- **variables/**  
  Stores all variable information.

- **variableslayer/**  
  Stores information about variable modes applied to layers. It allows retrieving variable values applied to layers and updating the variable set mode.

- **variablesset/**  
  Stores all variable set information.

#### `resources/`
Stores static resources (such as images, fonts, and basic data types). For example, style data like fill types are stored in `color.json`.

### `README.md`
The project documentation, introducing the directory structure and code structure.

---

## Code Explanation
1. The coding convention follows the component + instance approach. When a layer is recognized as an instance during generation, it references the related component to keep the code concise and improve reusability.

2. In the generated code, some properties may be variables or style modes, The rules for styles and variables can be referenced from the design draft.

3. During the conversion from design draft to code, complex vector layers or complex visual forms (e.g., an image using a clipping mode) will be converted into images.

4. We support complex prototype interactions, such as page navigation, mouse events, click events, and scroll direction support. More prototype interactions are being supported in future updates.

5. If compilation errors occur in the generated code, please contact product support. We will provide a remote solution.
