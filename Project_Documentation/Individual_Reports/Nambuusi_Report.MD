# SEDENTARY ACTIVITY TRACKER

**A Real-Time System for Monitoring Sedentary Behavior**

**Course:** Health Informatics  
**Module:** Media Management  
**Group Name:** Sedex  
**Student Name:** Nambuusi Emily Babirye  
**Mat.number:** 22405284  
**Semester:** Winter Semester 3

---

## 1. Introduction: Physiological Problem Addressed

Sedentary behavior refers to activities that involve very low energy expenditure while sitting, reclining, or lying down during waking hours. In modern academic and professional environments especially those requiring extensive digital interaction individuals often remain seated for long periods while mentally engaged with screens. University students, office employees, and other knowledge workers frequently spend hours in such postures, posing significant health challenges.

Scientific research links prolonged sedentary time to increased risks of musculoskeletal pain, poor blood circulation, metabolic disorders, and cardiovascular diseases (Owen et al., 2010; Thyfault & Booth, 2011). These risks exist independently of overall physical activity; even physically active individuals experience negative effects if they sit continuously without regular movement breaks.

One major difficulty is that people tend to underestimate their sedentary time because minor actions, such as fidgeting or shifting posture, may feel like adequate activity. However, physiological evidence shows that sustained stillness suppresses metabolism and vascular function, whereas frequent small movements can mitigate some harmful effects (Hamilton et al., 2008).

The Sedentary Activity Tracker addresses this issue by detecting low level physical inactivity during typical desk based tasks. It focuses on measuring subtle body movements rather than exercise intensity and converts this data into interactive visual feedback. From a media management perspective, the system conceptualizes biosignals as mediated content captured, processed, and displayed to promote self-awareness and behavioral change.

## 2. Technical Solution and System Architecture

### 2.1 System Overview

The Sedentary Activity Tracker follows a classic media pipeline model adapted for physiological signal processing. Raw biosignals pass through the stages of acquisition, preprocessing, feature extraction, classification, visualization, and feedback. This modular structure enables independent development and updates of the hardware, computational, and user interface components, ensuring both scalability and flexibility.

The system includes four main components:

- Sensor hardware: Captures continuous motion data.
- Signal processing backend: Filters and classifies signals.
- Data storage and analysis module: Maintains long-term records and detects patterns.
- Web-based dashboard: Visualizes data and provides user feedback.

This separation ensures simultaneous real-time response and offline data analysis.

### 2.2 Biosignal Acquisition and Sampling Strategy

An Arduino platform equipped with an MPU6050 accelerometer and a Passive Infrared (PIR) motion sensor was used. The accelerometer records three-axis acceleration data at 10 Hz—a sampling rate that balances temporal resolution and data efficiency while preventing aliasing and maintaining signal integrity.

Raw data are preprocessed to minimize noise and outliers through smoothing and filtering methods, providing stable and meaningful input for classification and analysis.

### 2.3 Feature Extraction and Classification Logic

Preprocessed acceleration signals are divided into short time windows. Features focus on acceleration differences within each window, emphasizing motion intensity and variability instead of absolute values.

Each window is classified into one of three categories:

- Active: Significant motion.
- Fidgeting: Small seated movements.
- Sedentary: Minimal or no movement.

A rule-based thresholding approach was implemented to ensure transparency, allowing users and developers to understand classification decisions without relying on opaque ("black box") algorithms. In addition, a sedentary timer tracks cumulative inactivity and issues feedback alerts after 20 consecutive minutes of stillness, aligning with recognized health guidelines (Torbeyns et al., 2014).

### 2.4 Backend Signal Processing and Data Architecture

The backend operates two parallel data streams:

- Real-time stream: Sends sensor data directly to the dashboard via WebSocket for low-latency feedback.
- Analysis stream: Stores data asynchronously in a PostgreSQL database following HL7 FHIR standards, ensuring interoperability with other health systems.

This dual-stream architecture effectively separates immediate user interaction from long-term archival analysis.

### 2.5 Visualization and Exploratory Data Analysis

The web dashboard displays biosignals as time-series graphs, activity indicators, and sedentary timers. This presentation transforms raw numerical data into intuitive visual information, encouraging users to interpret and reflect on their activity levels.

Additionally, unsupervised clustering is applied to detect recurring sedentary patterns across long-term data, such as afternoon inactivity peaks. This exploratory analysis prioritizes pattern recognition and self reflection over predictive modeling, helping promote awareness rather than prescriptive control.

## 3. Results and Analysis

Tests conducted in natural study and office conditions confirmed the system's ability to reliably distinguish between active, fidgeting, and sedentary states. Incorporating PIR data improved classification accuracy, particularly for subtle movements.

The system handled thousands of data points per session in real time without lag or data loss. Alerts were consistently triggered after 20 minutes of continuous inactivity. While this project did not formally evaluate performance metrics, operational testing demonstrated strong functional reliability.

User feedback showed increased awareness participants frequently adjusted their posture or took breaks prompted by dashboard visualization even before alerts were triggered. This response demonstrates the system's potential to use mediated feedback as a behavioral influence tool.

From a media management viewpoint, the system exemplifies how physiological data can serve as both content and communication medium, promoting user reflection and voluntary self-regulation rather than automated enforcement.

## 4. Individual Responsibility, Learnings, and Challenges

In this project, I was responsible for the machine learning component of the Sedentary Activity Tracker, effectively serving as the "brain" of the system. My work focused on ensuring that high-frequency sensor data from the Arduino could be accurately interpreted to classify user activity in real-time. I took ownership of the ml_classification/ folder and the sedentary_tracker_data.json dataset, ensuring all data and code for model training were properly organized and documented.

I wrote Python scripts to clean, normalize, and preprocess the JSONL sensor data, removing noise and gravitational effects to produce reliable inputs for machine learning. Using Pandas and Scikit-learn, I developed Random Forest and LSTM classifiers capable of detecting user states—Active, Fidgeting, and Sedentary. To integrate the ML component with the Rust backend, I built a Flask/FastAPI microservice (service.py) that accepts incoming [x, y, z] sensor arrays and returns predicted activity classes in JSON format. Additionally, I created a Dockerfile to containerize the service, allowing seamless deployment and integration into the system's Docker Compose network.

Through this work, I learned how to manage high-frequency, noisy time-series data, develop effective preprocessing pipelines, and train classifiers for real-time activity recognition. I also gained experience in building Python microservices, designing clear APIs, and integrating ML services into a multi-service architecture. A significant challenge was ensuring that the model could provide accurate predictions quickly enough for real-time feedback without overloading the backend. Balancing prediction speed, model complexity, and deployment reliability required careful testing, iterative improvements, and thoughtful containerization.

Overall, my contribution to the ML component ensured that the system could intelligently interpret sensor data and provide meaningful real-time feedback. This work strengthened my skills in data preprocessing, model development, microservice design, and containerized deployment, demonstrating how machine learning can be applied effectively in wearable health monitoring systems.

## 5. Conclusion

The Sedentary Activity Tracker illustrates how basic bio signal processing combined with transparent classification and interactive visualization can create effective health oriented media systems. By translating subtle body movements into meaningful digital feedback, the system encourages users to recognize and manage sedentary behavior.

Our findings highlight the importance of clear communication and user engagement in data driven health applications. This project bridges technical signal analysis with media management concepts, showing how technology can mediate bodily awareness and support healthier daily habits.

---

**Footnote**

Source code repository (Sedex group repository):  
https://github.com/Derrick-Projects/Sedentary_Tracker_Behaviour.git  
https://github.com/Derrick-Projects/Sedentary_Tracker_Behaviour.git

## References

Hamilton, M. T., Healy, G. N., Dunstan, D. W., Zderic, T. W., & Owen, N. (2008). Too little exercise and too much sitting: Inactivity physiology and the need for new recommendations on sedentary behavior. *Current Cardiovascular Risk Reports*, 2(4), 292–298. https://doi.org/10.1007/s12170-008-0054-8

HL7 International. (2024). FHIR release 5: Fast Healthcare Interoperability Resources. https://hl7.org/fhir/

Michie, S., van Stralen, M. M., & West, R. (2011). The behavior change wheel: A new method for characterizing and designing behavior change interventions. *Implementation Science*, 6, Article 42. https://doi.org/10.1186/1748-5908-6-42

Owen, N., Bauman, A., & Brown, W. (2010). Too much sitting: A novel and important predictor of chronic disease risk? *British Journal of Sports Medicine*, 43(2), 81–83. https://doi.org/10.1136/bjsm.2008.055269

Thyfault, J. P., & Booth, F. W. (2011). Lack of regular exercise or too much sitting: Separate and distinct health issues? *Journal of Applied Physiology*, 111(4), 1002–1006. https://doi.org/10.1152/japplphysiol.00365.2011

Torbeyns, T., Bailey, S., Bos, I., & Meeusen, R. (2014). Active workstations to fight sedentary behaviour. *Sports Medicine*, 44(9), 1261–1273. https://doi.org/10.1007/s40279-014-0202-x

Michie, S., van Stralen, M. M., & West, R. (2011). The behavior change wheel: A new method for characterizing and designing behavior change interventions. *Implementation Science*, 6(42).