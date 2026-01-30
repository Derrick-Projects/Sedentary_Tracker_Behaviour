# SEDENTARY ACTIVITY TRACKER

**A Real-Time System for Monitoring Sedentary Behavior**

**Course:** Health Informatics  
**Module:** Media Management  
**Group Name:** Sedex  
**Student Name:** Rebecca Atem Nyandeng  
**Matriculation Number:** 22410646  
**Semester:** Winter Semester 3

---

## 1. Introduction: Physiological Problem Addressed

Sedentary behavior can be defined as activities involving very low energy expenditure while sitting, reclining, or lying down during waking hours. In modern academic and professional contexts especially those involving extensive digital media interaction people often remain seated for long durations while mentally engaged with screens. University students, office employees, and other knowledge workers frequently spend hours continuously in sedentary postures, creating a significant health challenge.

Scientific studies link prolonged sedentary time to increased risks of musculoskeletal pain, impaired blood circulation, metabolic disorders, and cardiovascular disease (Owen et al., 2010; Thyfault & Booth, 2011). These risks are independent of overall physical activity, meaning even active individuals face negative effects if they sit for long periods without interruption.

A key difficulty is that people often underestimate their sedentary time because small movements like fidgeting or shifting posture feel like sufficient activity. Yet, physiological evidence shows sustained stillness suppresses metabolism and vascular function, while frequent small motions can alleviate some adverse effects (Hamilton et al., 2008).

Our Sedentary Activity Tracker addresses this issue by detecting low-level physical inactivity during typical desk-based activities. It focuses on measuring subtle body movements rather than exercise output and translates this data into interactive visual feedback. From a media management standpoint, the system conceptualizes biosignals as a form of mediated content, captured and presented back to the user to encourage self-awareness and behavioral adjustment.

## 2. Technical Solution and System Architecture

### 2.1 System Overview

The Sedentary Activity Tracker is structured around a classic media pipeline adapted for physiological signal processing. Raw biosignals flow through stages of acquisition, preprocessing, feature extraction, classification, visualization, and feedback. This modular architecture allows independent development and updates of hardware, computation, and user interface components, ensuring scalability and flexibility.

The system includes four main components:

- Sensor hardware: captures continuous motion data
- Signal processing backend: filters and classifies signals
- Data storage and analysis module: maintains long-term records and pattern detection
- Web-based dashboard: visualizes data and provides user feedback

This separation supports real-time response and offline analysis simultaneously.

### 2.2 Biosignal Acquisition and Sampling Strategy

We employed an Arduino platform equipped with an MPU6050 accelerometer and a Passive Infrared (PIR) motion sensor. The accelerometer continuously records three-axis (X, Y, Z) acceleration data at 10 Hz, hence determine the body's inclination, tilt and orientation. A modern sensor usually takes a sampling rate chosen to balance temporal resolution and data efficiency, following principles that avoid aliasing (signal distortion) and preserve signal integrity.

Raw data are preprocessed to reduce noise and outliers through smoothing and filtering techniques. This step ensures stable, meaningful input for classification and analysis.

### 2.3 Feature Extraction and Classification Logic

Preprocessed acceleration data are segmented into short windows. Features focus on acceleration differences within each window, reflecting motion intensity and variability rather than absolute values.

The system classifies each window into one of three categories:

#### 2.3.1 Active (significant motion)

This stage includes ambulatory activities such as running, climbing stairs or walking. These activities signify high amplitude and strong periodicity. As the feet hit the ground level, the signal illustrates rhythmic peaks and valleys. The key features include:

- High Variance/Standard Deviation: The values of the signal spread far from the mean, hence a result of intensity movement being high.
- Signal Magnitude Area (SMA): When the sum of the absolute values across all the three axes (X, Y, Z) is high, there is significant energy expenditure.
- Frequency Domain: Dominant frequencies appear in the range of 1 to 5 Hz (This is obviously human stepping).

#### 2.3.2 Fidgeting (small, seated movements)

This stage consists of non-ambulatory movements like shifting weight while sitting or tapping feet. These signals are random rather than periodic because they lack the consistent rhythm of walking. The features under this stage include:

- Short Bursts: These include zero-crossing rate which detects the rapid small movements that are too short to be considered as a proper exercise.
- Intensity: Fidgeting registers low to moderate intensity usually 1.5-2.9 METs (Metabolic Equivalents) hence definitely a non-zero but does not reach the threshold for light-intensity physical activity.
- Variability: Coefficient of Variation (CV) is used by trackers to distinguish between fidgeting and steady walking. Fidgeting is said to be much more unpredictable and varied than a smooth, rhythmic walk.

#### 2.3.3 Sedentary (minimal or no motion)

Here, sitting or lying down are covered. There is no muscular engagement. The sensor signal in this stage is almost entirely flat. Readings are dominated by the constant force of gravity which is approximately 9.8 m/s² rather unlike the dynamic acceleration. The key features are as below.

Minimal Activity Counts: Modern trackers usually classify sedentary time as any window where the movement count is less than 100 counts per minute.

Elevated Regularity: Due to extreme signal predictability and static, features such as Sample Entropy are too low.

Postural Clues: These include the advanced sensors like those worn on thighs which use feature extraction to analyse posture relative to gravity. They check the y-axis angle to determine whether the leg is horizontal (lying) or vertical (standing).

We used rule-based thresholding to maintain transparency, enabling users and developers to understand classification decisions without opaque (black box) algorithms. Alongside classification, a sedentary timer tracks cumulative inactivity and triggers feedback alerts after 20 minutes, consistent with health guidelines (Torbeyns et al., 2014).

### 2.4 Backend Signal Processing and Data Architecture

The backend operates two parallel streams:

- A real-time path streams sensor data immediately to the dashboard via WebSocket for low-latency feedback.
- An analysis path stores data asynchronously in a PostgreSQL database formatted according to HL7 FHIR standards, supporting interoperability with health systems.

This dual-stream design separates immediate user interaction from archival analysis.

### 2.5 Visualization and Exploratory Data Analysis

The web dashboard visualizes biosignals as time-series graphs, state indicators, and sedentary timers. This converts raw numerical data into intuitive visual media, encouraging users to interpret and reflect on their activity.

Additionally, unsupervised clustering identifies recurring sedentary patterns in long-term data, such as afternoon inactivity peaks. This exploratory approach focuses on structure detection rather than predictive accuracy, supporting reflective awareness rather than prescriptive intervention.

## 3. Results and Analysis of the Sedentary Activity Tracker

The system's operational testing yielded results that align with established research on human activity classification:

**Reliability in Natural Settings:** Testing under natural study and office conditions such as desks, computers among others confirmed the system's ability to distinguish between active, fidgeting, and sedentary states reliably. Incorporating PIR data improved classification accuracy, especially during subtle movements, enhancing the system's ability to interpret tricky signals near the less than or equal to 1.5 MET threshold.

**Data Processing and Alert Mechanism:** The system processed thousands of data points per session in real time, maintaining high signal regularity without lag or data loss. Alerts consistently activated after 20 minutes of continuous inactivity, confirming adherence to the criteria for extended sedentary periods.

**Functional Reliability and User Feedback:** While formal performance metrics (like specific sample entropy or zero-crossing rates) were outside this study's scope, operational testing demonstrated functional reliability. User feedback indicated heightened awareness; participants often adjusted posture or took breaks prompted by dashboard visualization before alerts occurred, evidencing the influence of mediated feedback on behavior.

**Behavioral Implications:** From a media management perspective, the results exemplify how raw physiological data can be transformed into communicative medium that fosters user reflection and self-regulation rather than simply imposing automated control based on hard thresholds.

## 4. Individual Responsibility, Learnings, and Challenges

Firstly, my primary role focused on backend development, where I designed and implemented the entire data pipeline and server-side logic necessary for processing and storing biosignals. This included developing the core classification engine, implementing the noise filtering and engineering the sedentary timer logic. I as well ensured the overall system integration and testing procedures to ensure all components communicated seamlessly. I also drafted the research report however much all the group members were involved. I also implemented security best practices like data encryption and authentication measures, to protect sensitive user health information.

Secondly, this project greatly deepened my practical understanding of the signal processing concepts like sampling theory, noise reduction and temporal segmentation. I applied these to real-world datasets, solidifying the importance of transparency and interpretable classification methods in health media systems. I have become quite good at writing neat code, consistent and easy to use again as well as ensuring that I documented it good so that it is simple to maintain and work on with the team. I also gained confidence working with different individuals having different ideas and interpersonal skills.

However, I faced a technical problem of calibrating the threshold values needed to balance sensitivity and specificity for different movements. Making thresholds too sensitively resulted to false positives for normal motion, while too strict caused the system to miss subtle fidgeting. This took much time due to several testing and iterations to find optimal parameters. I also faced issues of integrating different sensors and systems with varying data exchange formats. I got a limitation of keeping the system secure and private. I had to keep up with all the cybersecurity threats and make sure I had strong data protection to keep our information safe from any data breaches. I also faced a problem with scalability as it demanded careful planning for database partitioning and resource allocation because the system was designed to handle unexpected changes in both user and data volume. I as well overcame a challenge in Rust as it required me to master many concepts.

## 5. Conclusion

The Sedentary Activity Tracker demonstrates how simple biosignal processing combined with transparent classification and interactive visualization can create effective media systems for health awareness. By translating subtle body movement into meaningful digital feedback, the system supports users in recognizing and managing sedentary behavior.

Our results underline the value of clear communication and user engagement in media-driven health tools. This project bridges technical signal analysis with media management principles, illustrating how data-driven systems can mediate bodily experience and promote healthier habits.

---

**Footnote**

Source code repository (Sedex group repository):  
https://github.com/rebecca-dotcom/Sedex-Sedentary-Tracker.git

## References

Hamilton, M. T., Healy, G. N., Dunstan, D. W., Zderic, T. W., & Owen, N. (2008). Too little exercise and too much sitting: Inactivity physiology and the need for new recommendations on sedentary behavior. *Current Cardiovascular Risk Reports*, 2(4), 292–298. https://doi.org/10.1007/s12170-008-0054-8

HL7 International. (2024). FHIR release 5: Fast Healthcare Interoperability Resources. https://hl7.org/fhir/

Michie, S., van Stralen, M. M., & West, R. (2011). The behavior change wheel: A new method for characterizing and designing behavior change interventions. *Implementation Science*, 6, Article 42. https://doi.org/10.1186/1748-5908-6-42

Owen, N., Bauman, A., & Brown, W. (2010). Too much sitting: A novel and important predictor of chronic disease risk? *British Journal of Sports Medicine*, 43(2), 81–83. https://doi.org/10.1136/bjsm.2008.055269

Thyfault, J. P., & Booth, F. W. (2011). Lack of regular exercise or too much sitting: Separate and distinct health issues? *Journal of Applied Physiology*, 111(4), 1002–1006. https://doi.org/10.1152/japplphysiol.00365.2011

Torbeyns, T., Bailey, S., Bos, I., & Meeusen, R. (2014). Active workstations to fight sedentary behaviour. *Sports Medicine*, 44(9), 1261–1273. https://doi.org/10.1007/s40279-014-0202-x

Michie, S., van Stralen, M. M., & West, R. (2011). The behavior change wheel: A new method for characterizing and designing behavior change interventions. *Implementation Science*, 6(42).